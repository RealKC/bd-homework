use axum::{routing::get, Router};
use sqlx::SqlitePool;

mod books;
mod error;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let pool = SqlitePool::connect("sqlite://data.sqlite?mode=rwc")
        .await
        .expect("Failed to open database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let app = Router::new()
        .route("/", get(root))
        .route("/books", get(books::books))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello BD project"
}
