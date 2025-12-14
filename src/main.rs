#![warn(clippy::all)]
use clap::Parser;
use config::Config;
use handle_errors::return_error;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{Filter, http::Method};

mod store;
use store::Store;
mod profanity;
mod routes;
mod types;
use routes::answer::add_answer;
use routes::question::{add_question, delete_question, get_questions, update_question};

use crate::routes::authentication::{auth, login, register};

#[derive(Parser, Debug, Default, serde::Deserialize, PartialEq)]
struct Args {
    log_level: String,
    database_host: String,
    database_port: u16,
    database_name: String,
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let config = Config::builder()
        .add_source(config::File::with_name("setup"))
        .build()
        .unwrap();
    let config = config.try_deserialize::<Args>().unwrap();

    let log_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        format!(
            "handle_errors={},q_and_a={},warp={}",
            config.log_level, config.log_level, config.log_level
        )
    });

    // if you need to add a username and password,
    // the connection would look like:
    // "postgres://username:password@localhost:5432/q_and_a"
    let store = Store::new(&format!(
        "postgres://{}:{}/{}",
        config.database_host, config.database_port, config.database_name
    ))
    .await?;

    sqlx::migrate!().run(&store.clone().connection).await?;

    let store_filter = warp::any().map(move || store.clone());

    tracing_subscriber::fmt()
        // Use the filter we built above to determine which traces to record.
        .with_env_filter(log_filter)
        // Record an event when each span closes.
        // This can be used to time our
        // routes' durations!
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]);

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(get_questions)
        .with(warp::trace(|info| {
            tracing::info_span!(
                "get_questions_request",
                method = %info.method(),
                path = %info.path(),
                id = %uuid::Uuid::new_v4(),
            )
        }));

    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(add_question);

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(update_question);

    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(auth())
        .and(store_filter.clone())
        .and_then(delete_question);

    let add_answer = warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(auth())
        .and(store_filter.clone())
        .and(warp::body::form())
        .and_then(add_answer);

    let registration = warp::post()
        .and(warp::path("registration"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(register);

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(login);

    let routes = get_questions
        .or(add_question)
        .or(update_question)
        .or(delete_question)
        .or(add_answer)
        .or(registration)
        .or(login)
        .with(cors)
        .with(warp::trace::request())
        .recover(return_error);

    warp::serve(routes).run(([127, 0, 0, 1], config.port)).await;
    Ok(())
}
