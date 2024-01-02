use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use schema::{
    auth::{
        CreateAccount, DeleteUserReply, DeleteUserRequest, GetAllUsersReply, GetAllUsersRequest,
        Login, LoginReply, PromoteUserRequest, User,
    },
    Integer,
};
use sqlx::SqlitePool;

use crate::{
    error::{IntoRouteError, RouteError},
    utils::verify_user_is_librarian,
};

pub fn router(state: SqlitePool) -> Router<SqlitePool> {
    Router::new()
        .route("/login", post(login))
        .route("/create-account", post(create_account))
        .route("/all-users", post(get_all_users))
        .route("/delete-user", post(delete_user))
        .route("/promote-user", post(promote_user))
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
    verify_user_is_librarian(&pool, data.cookie).await?;

    let records = sqlx::query!(
        r#"
SELECT
    u.user_id,
    u.name,
    u.email,
    u.type,
    (SELECT COUNT(*) FROM Borrows bo WHERE bo.user_id = u.user_id) AS "borrowed_book_count!: i64"
FROM Users u;
    "#
    )
    .fetch_all(&pool)
    .await
    .http_status_error(StatusCode::INTERNAL_SERVER_ERROR)?;

    let reply = records
        .into_iter()
        .map(|record| User {
            id: record.user_id.unwrap(),
            name: record.name,
            email: record.email,
            kind: record.r#type,
            borrowed_book_count: record.borrowed_book_count,
        })
        .collect();

    Ok(Json(reply))
}

pub async fn promote_user(
    State(pool): State<SqlitePool>,
    Json(data): Json<PromoteUserRequest>,
) -> Result<(), RouteError> {
    verify_user_is_librarian(&pool, data.cookie).await?;

    sqlx::query!(
        r"
UPDATE Users
SET type = 2
WHERE user_id = ?
    ",
        data.user_to_be_promoted
    )
    .execute(&pool)
    .await
    .http_internal_error("Failed to execute UPDATE")?;

    Ok(())
}

pub async fn delete_user(
    State(pool): State<SqlitePool>,
    Json(data): Json<DeleteUserRequest>,
) -> Result<Json<DeleteUserReply>, RouteError> {
    verify_user_is_librarian(&pool, data.cookie.clone()).await?;

    if data.user_to_be_deleted == data.cookie.id {
        return Ok(Json(DeleteUserReply::CannotDeleteSelf));
    }

    let count = count_books_borrowed_by(&pool, data.user_to_be_deleted).await?;

    if count > 0 {
        return Ok(Json(DeleteUserReply::UsersStillHadBooks));
    }

    sqlx::query!(
        r"
DELETE FROM Users
WHERE user_id = ?
    ",
        data.user_to_be_deleted
    )
    .execute(&pool)
    .await
    .http_internal_error("Failed to execute DELETE")?;

    Ok(Json(DeleteUserReply::Ok))
}

async fn count_books_borrowed_by(pool: &SqlitePool, user_id: Integer) -> Result<u64, RouteError> {
    let record = sqlx::query!(
        r#"
SELECT COUNT(*) as "count"
FROM Borrows
WHERE user_id = ?
    "#,
        user_id
    )
    .fetch_one(pool)
    .await
    .http_internal_error("Failed to count books")?;

    Ok(record.count as u64)
}
