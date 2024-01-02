use axum::http::StatusCode;
use schema::{session, LIBRARIAN};
use sqlx::SqlitePool;

use crate::error::{IntoRouteError, RouteError};

pub async fn verify_user_is_librarian(
    pool: &SqlitePool,
    cookie: session::Cookie,
) -> Result<(), RouteError> {
    let requester_id = cookie.id;
    let requester_type = sqlx::query!("SELECT type FROM Users WHERE user_id = ?", requester_id)
        .fetch_one(pool)
        .await
        .http_status_error(StatusCode::INTERNAL_SERVER_ERROR)?;

    if requester_type.r#type != LIBRARIAN {
        return Err(RouteError::new_forbidden());
    }

    Ok(())
}
