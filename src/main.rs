mod contact;
mod github;

use crate::{contact::contact_me, github::GitHub};

use anyhow::Result;
use axum::{
    extract::Path,
    handler::{get, post, Handler},
    http::StatusCode,
    response::IntoResponse,
    AddExtensionLayer, Json, Router,
};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgConnectOptions, ConnectOptions, PgPool, Pool, Postgres};
use std::{convert::Infallible, net::SocketAddr, str::FromStr, time::Duration};
use tower::{BoxError, ServiceBuilder};
use tower_http::{
    compression::CompressionLayer, decompression::DecompressionLayer, trace::TraceLayer,
};

pub type DbPool = Pool<Postgres>;

#[derive(Debug, Deserialize, Clone)]
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

async fn hello(Path(name): Path<String>) -> String {
    format!("Hello, {}!", name)
}

#[derive(Serialize)]
struct ErrorResponse {
    reason: &'static str,
    status: u16,
}

async fn not_found() -> impl IntoResponse {
    let status_code = StatusCode::NOT_FOUND;
    let response = Json(ErrorResponse {
        reason: status_code.canonical_reason().unwrap_or_default(),
        status: status_code.as_u16(),
    });

    (status_code, response)
}

async fn initialize_db(env: &Env) -> Result<DbPool> {
    let mut opts = PgConnectOptions::from_str(&env.database_url)?;
    opts.disable_statement_logging();

    let pool = PgPool::connect_with(opts).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "personal_api=debug,tower_http=debug");
    }

    tracing_subscriber::fmt::init();
    dotenv().ok();

    let env = envy::from_env::<Env>()?;
    let pool = initialize_db(&env).await?;

    if std::env::var("LOCAL").is_ok() {
        GitHub::new(&env.login, &env.token).await?;
    }

    let app = Router::new()
        .route("/hello/:name", get(hello))
        .route("/contact", post(contact_me))
        .layer(
            ServiceBuilder::new()
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(DecompressionLayer::new())
                .layer(AddExtensionLayer::new(pool))
                .layer(AddExtensionLayer::new(env))
                .into_inner(),
        )
        .handle_error(|error: BoxError| {
            let result = if error.is::<tower::timeout::error::Elapsed>() {
                Ok(StatusCode::REQUEST_TIMEOUT)
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                ))
            };

            Ok::<_, Infallible>(result)
        })
        .check_infallible();

    let app = app.or(not_found.into_service());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("Listening on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
