use axum::{routing::get, Router};
use sqlx::SqlitePool;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let pool = SqlitePool::connect("sqlite:data.sqlite")
        .await
        .expect("Failed to open database");

    let app = Router::new().route("/", get(root)).with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello BD project"
}
