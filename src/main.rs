use dotenv::dotenv;
use pretty_env_logger;
use warp::Filter;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    warp::serve(hello).run(([0, 0, 0, 0], 8080)).await;
}
