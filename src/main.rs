#[macro_use]
extern crate rocket;

pub mod github;

use anyhow::Result;
use dotenv::dotenv;
use rocket::{Build, Rocket};
use serde::Deserialize;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

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
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&env.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![hello])
}

#[rocket::main]
async fn main() {
    dotenv().ok();

    let env = match envy::from_env::<Env>() {
        Ok(env) => env,
        Err(e) => panic!("{}", e),
    };

    let _pool = match initialize_db(&env).await {
        Ok(pool) => pool,
        Err(e) => panic!("{}", e),
    };

    let mut gh = GitHub::new();
    gh.update(&env.login, &env.token).await.unwrap();

    if let Err(e) = rocket().launch().await {
        eprintln!("Rocket could not launch: {}", e);
        drop(e);
    }
}
