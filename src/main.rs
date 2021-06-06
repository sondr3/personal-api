pub mod github;

use anyhow::Result;
use dotenv::dotenv;
use log::info;
use serde::Deserialize;
use warp::Filter;

use crate::github::GitHub;

#[derive(Debug, Deserialize)]
struct Env {
    login: String,
    token: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    pretty_env_logger::init();

    let env = envy::from_env::<Env>()?;
    info!("{:?}", env);

    let mut gh = GitHub::new();
    gh.update(&env.login, &env.token).await?;

    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    warp::serve(hello).run(([0, 0, 0, 0], 8080)).await;

    Ok(())
}
