use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use schema::{
    auth::{CreateAccount, GetAllUsersReply, GetAllUsersRequest, Login, LoginReply, User},
    LIBRARIAN,
};
use sqlx::SqlitePool;

use crate::error::{IntoRouteError, RouteError};

pub fn router(state: SqlitePool) -> Router<SqlitePool> {
    Router::new()
        .route("/login", post(login))
        .route("/create-account", post(create_account))
        .route("/all-users", post(get_all_users))
        .with_state(state)
}

async fn login(
    State(pool): State<SqlitePool>,
    Json(data): Json<Login>,
) -> Result<Json<LoginReply>, RouteError> {
    let record = sqlx::query!(
        "
SELECT user_id, type, password
FROM Users
WHERE email = ?
    ",
        data.email
    )
    .fetch_optional(&pool)
    .await
    .http_error("No account with given email", StatusCode::NOT_FOUND)?;

    if let Some(user) = record {
        let parsed_hash = PasswordHash::new(&user.password)
            .http_status_error(StatusCode::INTERNAL_SERVER_ERROR)?;
        let password_is_valid = Argon2::default()
            .verify_password(data.password.as_bytes(), &parsed_hash)
            .is_ok();

        if password_is_valid {
            tracing::info!("Succesful login");

            Ok(Json(LoginReply {
                id: user.user_id.unwrap(),
                kind: user.r#type,
            }))
        } else {
            Err(RouteError::new_unauthorized())
        }
    } else {
        Err(RouteError::new_not_found())
    }
}

async fn create_account(
    State(pool): State<SqlitePool>,
    Json(data): Json<CreateAccount>,
) -> Result<Json<LoginReply>, RouteError> {
    if data.email.is_empty() || data.name.is_empty() || data.password.is_empty() {
        return Err(RouteError::new_bad_request());
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(data.password.as_bytes(), &salt)
        .http_status_error(StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    let record = sqlx::query!(
        "
INSERT INTO Users(name, type, email, password) VALUES (?, 1, ?, ?)
RETURNING user_id, type;
",
        data.name,
        data.email,
        password_hash
    )
    .fetch_one(&pool)
    .await
    .http_status_error(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(LoginReply {
        id: record.user_id,
        kind: record.r#type,
    }))
}

pub async fn get_all_users(
    State(pool): State<SqlitePool>,
    Json(data): Json<GetAllUsersRequest>,
) -> Result<Json<GetAllUsersReply>, RouteError> {
    let requester_id = data.cookie.id;
    let requester_type = sqlx::query!("SELECT type FROM Users WHERE user_id = ?", requester_id)
        .fetch_one(&pool)
        .await
        .http_status_error(StatusCode::INTERNAL_SERVER_ERROR)?;

    if requester_type.r#type != LIBRARIAN {
        return Err(RouteError::new_forbidden());
    }

    let records = sqlx::query!(
        r#"
SELECT user_id, name, email, type
FROM Users;
    "#
    )
    .fetch_all(&pool)
    .await
    .http_status_error(StatusCode::INTERNAL_SERVER_ERROR)?;

    let reply = records
        .into_iter()
        .map(|record| User {
            id: record.user_id,
            name: record.name,
            email: record.email,
            kind: record.r#type,
        })
        .collect();

    Ok(Json(reply))
}
