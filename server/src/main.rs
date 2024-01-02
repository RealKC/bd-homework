use std::str::FromStr;

use axum::{
    routing::{get, post},
    Router,
};
use error::RouteError;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};

mod auth;
mod books;
mod error;
mod utils;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let pool = SqlitePool::connect_with(
        SqliteConnectOptions::from_str("sqlite://data.sqlite?mode=rwc")
            .unwrap()
            .foreign_keys(true),
    )
    .await
    .expect("Failed to open database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let app = Router::new()
        .route("/", get(root))
        .route("/books", get(books::books))
        .route("/borrow", post(books::borrow))
        .route("/borrowed-by/:user_id", post(books::borrowed_by))
        .nest("/auth", auth::router(pool.clone()))
        .fallback(fallback)
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello BD project"
}

async fn fallback() -> RouteError {
    RouteError::new_not_found()
}
