#[macro_use]
extern crate rocket;

pub mod github;

use dotenv::dotenv;
use rocket::{
    fairing::{self, AdHoc},
    Build, Rocket,
};
use serde::Deserialize;
use sqlx::{sqlite::SqliteConnectOptions, ConnectOptions, SqlitePool};

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

async fn initialize_db(rocket: Rocket<Build>) -> fairing::Result {
    let env = match envy::from_env::<Env>() {
        Ok(env) => env,
        Err(e) => panic!("{}", e),
    };

    let mut opts = SqliteConnectOptions::new()
        .filename(&env.database_url)
        .create_if_missing(true);

    opts.disable_statement_logging();
    let pool = match SqlitePool::connect_with(opts).await {
        Ok(it) => it,
        Err(e) => panic!("{}", e),
    };

    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(_) => {}
        Err(e) => panic!("{}", e),
    };

    Ok(rocket.manage(pool))
}

fn rocket() -> Rocket<Build> {
    rocket::build()
        .attach(AdHoc::try_on_ignite("SQLx", initialize_db))
        .mount("/", routes![hello])
}

#[rocket::main]
async fn main() {
    dotenv().ok();

    let env = match envy::from_env::<Env>() {
        Ok(env) => env,
        Err(e) => panic!("{}", e),
    };

    let mut gh = GitHub::new();
    gh.update(&env.login, &env.token).await.unwrap();

    if let Err(e) = rocket().launch().await {
        eprintln!("Rocket could not launch: {}", e);
        drop(e);
    }
}
