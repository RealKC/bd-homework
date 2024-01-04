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
        .route("/authors", get(books::authors))
        .route("/books", get(books::books))
        .route("/borrow", post(books::borrow))
        .route("/borrows", post(books::borrows))
        .route("/borrowed-by/:user_id", post(books::borrowed_by))
        .route("/change-author-details", post(books::change_author_details))
        .route("/change-book-details", post(books::change_book_details))
        .route("/delete-book/:book_id", post(books::delete_book))
        .route(
            "/lengthen-borrow/:borrow_id",
            post(books::lengthen_borrow_by),
        )
        .route("/end-borrow/:borrow_id", post(books::end_borrow))
        .route(
            "/update-borrow-chapters-read/:borrow_id",
            post(books::update_chapters_read),
        )
        .route("/return-book/:borrow_id", post(books::return_book))
        .nest("/auth", auth::router(pool.clone()))
        .fallback(fallback)
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn fallback() -> RouteError {
    RouteError::new_not_found()
}
