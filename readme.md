# PostgresDB Install

```
brew install postgresql@14

brew services start postgresql@14

psql postgres

create database q_and_a;

List all databases -> \l

Quit -> \q


CREATE TABLE IF NOT EXISTS questions (
    id serial PRIMARY KEY,
    title VARCHAR (255) NOT NULL,
    content TEXT NOT NULL,
    tags TEXT [],
    created_on TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS answers (
    id serial PRIMARY KEY,
    content TEXT NOT NULL,
    created_on TIMESTAMP NOT NULL DEFAULT NOW(),
    corresponding_question integer REFERENCES questions
);


List tables -> \dt

drop table answers, questions;
\dt

sqlx migrate add -r questions_table
sqlx migrate run --database-url postgresql://localhost:5432/q_and_a
sqlx migrate revert --database-url postgresql://localhost:5432/q_and_a

sqlx migrate add -r create_accounts_table

CREATE TABLE IF NOT EXISTS accounts (
    id serial NOT NULL,
    email VARCHAR(255) NOT NULL PRIMARY KEY,
    password VARCHAR(255) NOT NULL
);

DROP TABLE IF EXISTS accounts;

sqlx migrate add -r extend_questions_table;
sqlx migrate add -r extend_answers_table;

ALTER TABLE questions
ADD COLUMN account_id serial;

ALTER TABLE questions
DROP COLUMN account_id;

ALTER TABLE answers
ADD COLUMN account_id serial;

ALTER TABLE answers
DROP COLUMN account_id;

cargo run -- --db-host localhost --log-level warn --db-name q_and_a --db-port 5432 --port 8080

```
