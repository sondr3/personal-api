#[macro_use]
extern crate rocket;

mod contact;
mod github;

use anyhow::Result;
use dotenv::dotenv;
use rocket::{Build, Rocket};
use serde::Deserialize;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous},
    ConnectOptions, Pool, Sqlite, SqlitePool,
};
use std::str::FromStr;

use crate::{contact::contact_me, github::GitHub};

#[derive(Debug, Deserialize)]
pub struct Env {
    login: String,
    token: String,
    whoami: String,
    contact_email: String,
    email: String,
    relay: String,
    smtp_user: String,
    smtp_pass: String,
    database_url: String,
}

#[get("/hello/<name>")]
fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

async fn initialize_db(env: &Env) -> Result<Pool<Sqlite>> {
    let mut opts = SqliteConnectOptions::from_str(&env.database_url)?
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .create_if_missing(true);
    opts.disable_statement_logging();

    let pool = SqlitePool::connect_with(opts).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

fn rocket(env: Env, pool: Pool<Sqlite>) -> Rocket<Build> {
    rocket::build()
        .manage(env)
        .manage(pool)
        .mount("/", routes![hello, contact_me])
}

#[rocket::main]
async fn main() {
    dotenv().ok();

    let env = match envy::from_env::<Env>() {
        Ok(env) => env,
        Err(e) => panic!("{}", e),
    };

    let pool = match initialize_db(&env).await {
        Ok(p) => p,
        Err(e) => panic!("{}", e),
    };

    if std::env::var("prod").is_ok() {
        let mut gh = GitHub::new();
        gh.update(&env.login, &env.token).await.unwrap();
    }

    if let Err(e) = rocket(env, pool).launch().await {
        eprintln!("Rocket could not launch: {}", e);
        drop(e);
    }
}
