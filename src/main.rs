#[macro_use]
extern crate rocket;

mod contact;
mod github;

use crate::{contact::contact_me, github::GitHub};

use anyhow::Result;
use dotenv::dotenv;
use rocket::{
    http::{Method, Status},
    serde::json::Json,
    Build, Request, Rocket,
};
use rocket_cors::{AllowedOrigins, CorsOptions};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgConnectOptions, ConnectOptions, PgPool, Pool, Postgres};
use std::str::FromStr;

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

#[derive(Serialize)]
struct ErrorResponse {
    reason: String,
    status: u16,
}

#[catch(404)]
fn not_found(_: &Request) -> Json<ErrorResponse> {
    Json(ErrorResponse {
        reason: "Not found".to_string(),
        status: Status::NotFound.code,
    })
}

#[catch(400)]
fn bad_request(_: &Request) -> Json<ErrorResponse> {
    Json(ErrorResponse {
        reason: "Bad request".to_string(),
        status: Status::BadRequest.code,
    })
}

#[catch(500)]
fn internal_error(_: &Request) -> Json<ErrorResponse> {
    Json(ErrorResponse {
        reason: "Internal server error".to_string(),
        status: Status::InternalServerError.code,
    })
}

#[catch(default)]
fn default_catcher(status: Status, _: &Request) -> Status {
    status
}

async fn initialize_db(env: &Env) -> Result<Pool<Postgres>> {
    let mut opts = PgConnectOptions::from_str(&env.database_url)?;
    opts.disable_statement_logging();

    let pool = PgPool::connect_with(opts).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

fn rocket(env: Env, pool: Pool<Postgres>) -> Rocket<Build> {
    let allowed_origins =
        AllowedOrigins::some_exact(&["https://www.eons.io", "http://localhost:3000"]);
    let cors = CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post]
            .into_iter()
            .map(From::from)
            .collect(),
        ..Default::default()
    }
    .to_cors()
    .unwrap();

    rocket::build()
        .attach(cors)
        .manage(env)
        .manage(pool)
        .register(
            "/",
            catchers![default_catcher, not_found, bad_request, internal_error],
        )
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
