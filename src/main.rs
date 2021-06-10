#[macro_use]
extern crate rocket;

pub mod github;

use anyhow::Result;
use dotenv::dotenv;
use rocket::{Build, Rocket};
use serde::Deserialize;
use sqlx::{sqlite::SqliteConnectOptions, ConnectOptions, Pool, Sqlite, SqlitePool};

use crate::github::GitHub;

#[derive(Debug, Deserialize)]
struct Env {
    login: String,
    token: String,
    database_url: String,
}

#[get("/hello/<name>")]
fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

async fn initialize_db(env: &Env) -> Result<Pool<Sqlite>> {
    let mut opts = SqliteConnectOptions::new()
        .filename(&env.database_url)
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
        .mount("/", routes![hello])
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

    let mut gh = GitHub::new();
    gh.update(&env.login, &env.token).await.unwrap();

    if let Err(e) = rocket(env, pool).launch().await {
        eprintln!("Rocket could not launch: {}", e);
        drop(e);
    }
}
