use clap::Parser;
use std::env;

/// Q&A web service API
#[derive(Parser, Debug, PartialEq)]
#[clap(author, version, about, long_about = None)]
pub struct Config {
    /// Which errors we want to log (info, warn or error)
    #[clap(short, long, default_value = "warn")]
    pub log_level: String,
    /// Which PORT the server is listening to
    #[clap(short, long, default_value = "8080")]
    pub port: u16,
    /// Database user
    #[clap(long, default_value = "username")]
    pub db_user: String,
    /// Database user
    #[clap(long, default_value = "password")]
    pub db_password: String,
    /// URL for the postgres database
    #[clap(long, default_value = "localhost")]
    pub db_host: String,
    /// PORT number for the database connection
    #[clap(long, default_value = "5432")]
    pub db_port: u16,
    /// Database name
    #[clap(long, default_value = "q_and_a")]
    pub db_name: String,
}

impl Config {
    pub fn new() -> Result<Config, handle_errors::Error> {
        let config = Config::parse();

        if let Err(_) = env::var("BAD_WORDS_API_KEY") {
            panic!("BadWords API key not set");
        }

        if let Err(_) = env::var("PASETO_KEY") {
            panic!("PASETO_KEY not set");
        }

        let port = std::env::var("PORT")
            .ok()
            .map(|val| val.parse::<u16>())
            .unwrap_or(Ok(config.port))
            .map_err(|e| handle_errors::Error::ParseError(e))?;

        let db_user = env::var("POSTGRES_USER").unwrap_or(config.db_user.to_owned());
        let db_password = env::var("POSTGRES_PASSWORD").unwrap();
        let db_host = env::var("POSTGRES_HOST").unwrap_or(config.db_host.to_owned());
        let db_port = env::var("POSTGRES_PORT").unwrap_or(config.db_port.to_string());
        let db_name = env::var("POSTGRES_DB").unwrap_or(config.db_name.to_owned());

        Ok(Config {
            log_level: config.log_level,
            port,
            db_user,
            db_password,
            db_host,
            db_port: db_port
                .parse::<u16>()
                .map_err(|e| handle_errors::Error::ParseError(e))?,
            db_name,
        })
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    fn set_env() {
        unsafe { env::set_var("BAD_WORDS_API_KEY", "yes") };
        unsafe {
            env::set_var("PASETO_KEY", "RANDOM WORDS WINTER MACINTOSH PC");
        };
        unsafe { env::set_var("POSTGRES_USER", "user") };
        unsafe { env::set_var("POSTGRES_PASSWORD", "password") };
        unsafe { env::set_var("POSTGRES_HOST", "localhost") };
        unsafe { env::set_var("POSTGRES_PORT", "5432") };
        unsafe { env::set_var("POSTGRES_DB", "q_and_a") };
    }

    fn unset_env() {
        unsafe { env::remove_var("BAD_WORDS_API_KEY") };
        unsafe { env::remove_var("PASETO_KEY") };
        unsafe { env::remove_var("POSTGRES_USER") };
        unsafe { env::remove_var("POSTGRES_PASSWORD") };
        unsafe { env::remove_var("POSTGRES_HOST") };
        unsafe { env::remove_var("POSTGRES_PORT") };
        unsafe { env::remove_var("POSTGRES_DB") };
    }

    #[test]
    fn unset_api_key() {
        unset_env();
        let result = std::panic::catch_unwind(|| Config::new());
        assert!(result.is_err());
    }

    #[test]
    fn unset_and_set_api_key() {
        // ENV Variables are not set
        let result = std::panic::catch_unwind(|| Config::new());
        assert!(result.is_err());

        // Now we set them
        set_env();

        let expected = Config {
            log_level: "warn".to_string(),
            port: 8080,
            db_user: "user".to_string(),
            db_password: "password".to_string(),
            db_host: "localhost".to_string(),
            db_port: 5432,
            db_name: "q_and_a".to_string(),
        };

        let config = Config::new().unwrap();

        assert_eq!(config, expected);
    }
}
