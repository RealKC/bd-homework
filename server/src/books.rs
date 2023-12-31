use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{Days, Local};
use schema::{
    books::{
        Author, Book, Borrow, BorrowReply, BorrowRequest, BorrowedBook, BorrowedByReply,
        BorrowsReply, BorrowsRequest, ChangeAuthorDetailsRequest, ChangeBookDetailsRequest,
    },
    session,
};
use sqlx::SqlitePool;

use crate::{
    error::{IntoRouteError, RouteError},
    utils::verify_user_is_librarian,
};

pub async fn books(State(pool): State<SqlitePool>) -> Result<Json<Vec<Book>>, RouteError> {
    let data = sqlx::query!(
        r#"
SELECT
b.book_id, b.title, b.publish_date, b.publisher, b.count, b.synopsis, b.language,
a.author_id, a.name, a.date_of_birth, a.date_of_death, a.description,
b.count > (SELECT COUNT(*) FROM Borrows bo WHERE bo.book_id = b.book_id) AS "can_be_borrowed"
FROM Books b JOIN Authors a ON b.author_id = a.author_id;
"#
    )
    .fetch_all(&pool)
    .await
    .http_internal_error("Failed to fetch book information")?
    .into_iter()
    .map(|record| Book {
        book_id: record.book_id.unwrap(),
        title: record.title,
        author: Author {
            author_id: record.author_id.unwrap(),
            name: record.name,
            date_of_birth: record.date_of_birth,
            date_of_death: record.date_of_death,
            description: record.description,
        },
        publish_date: record.publish_date,
        publisher: record.publisher,
        count: record.count,
        synopsis: record.synopsis,
        can_be_borrowed: record.can_be_borrowed.map(|c| c != 0).unwrap_or(false),
    })
    .collect::<Vec<_>>();

    Ok(Json(data))
}

pub async fn authors(State(pool): State<SqlitePool>) -> Result<Json<Vec<Author>>, RouteError> {
    let data = sqlx::query!(
        r#"
SELECT author_id, name
FROM Authors;
    "#
    )
    .fetch_all(&pool)
    .await
    .http_internal_error("Failed to fetch Authors")?
    .into_iter()
    .map(|record| Author {
        author_id: record.author_id,
        name: record.name,
        date_of_birth: 0,
        date_of_death: None,
        description: "".into(),
    })
    .collect();

    Ok(Json(data))
}

pub async fn borrow(
    State(pool): State<SqlitePool>,
    Json(request): Json<BorrowRequest>,
) -> Result<Json<BorrowReply>, RouteError> {
    let mut tx = pool
        .begin()
        .await
        .http_internal_error("Failed to begin transaction")?;

    let record = sqlx::query!(
        r#"
SELECT b.count > (SELECT COUNT(*) FROM Borrows bo WHERE bo.book_id = b.book_id) AS "can_be_borrowed!: bool"
FROM Books b
WHERE b.book_id = ?;
    "#,
        request.book_id
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();

    if record.can_be_borrowed {
        let record = sqlx::query!(
            r#"
SELECT COUNT(*) as "times_borrowed"
FROM Books b JOIN Borrows bo ON b.book_id = bo.book_id
             JOIN Users   u  ON bo.user_id = u.user_id
WHERE b.book_id = ? AND u.user_id = ?;
        "#,
            request.book_id,
            request.cookie.id
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();

        if record.times_borrowed >= 1 {
            return Ok(Json(BorrowReply {
                already_borrowed: true,
            }));
        }

        let valid_until = (Local::now() + Days::new(30)).timestamp();

        let borrow_id = sqlx::query!(
            r#"
INSERT INTO Borrows(book_id, user_id) VALUES (?, ?)
RETURNING borrow_id;
        "#,
            request.book_id,
            request.cookie.id
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();

        sqlx::query!(
            r#"
INSERT INTO BorrowData(borrow_id, valid_until, chapters_read)
VALUES (?, ?, 0);
    "#,
            borrow_id.borrow_id,
            valid_until
        )
        .execute(&mut *tx)
        .await
        .unwrap();
    }

    tx.commit().await.unwrap();

    Ok(Json(BorrowReply {
        already_borrowed: false,
    }))
}

pub async fn borrowed_by(
    Path(user_id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<Json<BorrowedByReply>, RouteError> {
    let records = sqlx::query!(
        r#"
SELECT d.borrow_id, b.book_id, d.valid_until, d.chapters_read
FROM Borrows b JOIN BorrowData d ON b.borrow_id = d.borrow_id
WHERE b.user_id = ?
    "#,
        user_id
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    Ok(Json(
        records
            .into_iter()
            .map(|record| BorrowedBook {
                borrow_id: record.borrow_id,
                book_id: record.book_id,
                valid_until: record.valid_until,
                chapters_read: record.chapters_read,
            })
            .collect(),
    ))
}

pub async fn borrows(
    State(pool): State<SqlitePool>,
    Json(request): Json<BorrowsRequest>,
) -> Result<Json<BorrowsReply>, RouteError> {
    verify_user_is_librarian(&pool, request.cookie).await?;

    let records = sqlx::query!(
        r"
SELECT b.borrow_id, b.book_id, b.user_id, d.valid_until
FROM Borrows b JOIN BorrowData d ON b.borrow_id = d.borrow_id;
    "
    )
    .fetch_all(&pool)
    .await
    .http_internal_error("Failed to fetch borrows")?;

    let response = records
        .into_iter()
        .map(|record| Borrow {
            borrow_id: record.borrow_id,
            book_id: record.book_id,
            user_id: record.user_id,
            valid_until: record.valid_until,
        })
        .collect();

    Ok(Json(response))
}

pub async fn change_book_details(
    State(pool): State<SqlitePool>,
    Json(request): Json<ChangeBookDetailsRequest>,
) -> Result<(), RouteError> {
    verify_user_is_librarian(&pool, request.cookie).await?;

    if let Some(book_id) = request.book_id {
        sqlx::query!(
            r#"
UPDATE Books SET
    title = ?,
    author_id = ?,
    publish_date = ?,
    publisher = ?,
    count = ?,
    synopsis = ?
WHERE book_id = ?
        "#,
            request.title,
            request.author_id,
            request.publish_date,
            request.publisher,
            request.count,
            request.synopsis,
            book_id
        )
        .execute(&pool)
        .await
        .http_internal_error("Failed to update book")?;
    } else {
        sqlx::query!(
            r#"
INSERT INTO Books(title, author_id, publish_date, publisher, count, synopsis, language)
VALUES (?, ?, ?, ?, ?, ?, 'ro')
        "#,
            request.title,
            request.author_id,
            request.publish_date,
            request.publisher,
            request.count,
            request.synopsis,
        )
        .execute(&pool)
        .await
        .http_internal_error("Failed to insert book")?;
    }

    Ok(())
}

pub async fn end_borrow(
    Path(borrow_id): Path<i64>,
    State(pool): State<SqlitePool>,
    Json(cookie): Json<session::Cookie>,
) -> Result<(), RouteError> {
    verify_user_is_librarian(&pool, cookie).await?;

    sqlx::query!(
        r#"
UPDATE BorrowData
SET valid_until = unixepoch()
WHERE borrow_id = ?
    "#,
        borrow_id
    )
    .execute(&pool)
    .await
    .http_status_error(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

pub async fn lengthen_borrow_by(
    Path(borrow_id): Path<i64>,
    Query(params): Query<HashMap<String, i64>>,
    State(pool): State<SqlitePool>,
    Json(cookie): Json<session::Cookie>,
) -> Result<(), RouteError> {
    verify_user_is_librarian(&pool, cookie).await?;

    let Some(days) = params.get("days") else {
        return Err(RouteError::new_bad_request());
    };

    sqlx::query!(
        r#"
UPDATE BorrowData
SET valid_until = unixepoch(valid_until, 'unixepoch', '+' || ? || ' days')
WHERE borrow_id = ?
    "#,
        days,
        borrow_id
    )
    .execute(&pool)
    .await
    .http_status_error(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

pub async fn delete_book(
    Path(book_id): Path<i64>,
    State(pool): State<SqlitePool>,
    Json(cookie): Json<session::Cookie>,
) -> Result<(), RouteError> {
    verify_user_is_librarian(&pool, cookie).await?;

    let mut transaction = pool
        .begin()
        .await
        .http_status_error(StatusCode::INTERNAL_SERVER_ERROR)?;

    let borrowed_count = sqlx::query!(
        r#"
SELECT COUNT(*) as "count"
FROM Borrows
WHERE book_id = ?;
    "#,
        book_id
    )
    .fetch_one(&mut *transaction)
    .await
    .http_internal_error("Failed to delete borrow 2")?;

    if borrowed_count.count > 0 {
        return Err(RouteError::new_forbidden());
    }

    sqlx::query!(
        "
DELETE FROM Books
WHERE book_id = ?;
    ",
        book_id
    )
    .execute(&mut *transaction)
    .await
    .http_internal_error("Failed to delete borrow 2")?;

    transaction
        .commit()
        .await
        .http_internal_error("Failed to delete borrow 3")?;

    Ok(())
}

pub async fn update_chapters_read(
    Path(borrow_id): Path<i64>,
    Query(params): Query<HashMap<String, i64>>,
    State(pool): State<SqlitePool>,
    Json(_): Json<session::Cookie>,
) -> Result<(), RouteError> {
    let Some(value) = params.get("value") else {
        return Err(RouteError::new_bad_request());
    };

    sqlx::query!(
        "
UPDATE BorrowData
SET chapters_read = ?
WHERE borrow_id = ?
    ",
        value,
        borrow_id
    )
    .execute(&pool)
    .await
    .http_internal_error("Failed to update chapters read")?;

    Ok(())
}

pub async fn change_author_details(
    State(pool): State<SqlitePool>,
    Json(request): Json<ChangeAuthorDetailsRequest>,
) -> Result<(), RouteError> {
    verify_user_is_librarian(&pool, request.cookie.clone()).await?;

    tracing::info!("Going to add a new author: {request:?}");

    if request.author_id.is_some() {
        return Err(RouteError::new_bad_request());
    }

    sqlx::query!(
        r#"
INSERT INTO Authors(name, date_of_birth, date_of_death, description)
VALUES(?, ?, ?, ?)
    "#,
        request.name,
        request.date_of_birth,
        request.date_of_death,
        request.description
    )
    .execute(&pool)
    .await
    .http_internal_error("Failed to add author")?;

    Ok(())
}

pub async fn return_book(
    Path(borrow_id): Path<i64>,
    State(pool): State<SqlitePool>,
    Json(_): Json<session::Cookie>,
) -> Result<(), RouteError> {
    let mut transaction = pool
        .begin()
        .await
        .http_internal_error("Failed to start transaction")?;

    sqlx::query!(
        "
DELETE FROM BorrowData
WHERE borrow_id = ?;
    ",
        borrow_id
    )
    .execute(&mut *transaction)
    .await
    .http_internal_error("Failed to delete data")?;

    sqlx::query!(
        "
DELETE FROM Borrows
WHERE borrow_id = ?;
    ",
        borrow_id
    )
    .execute(&mut *transaction)
    .await
    .http_internal_error("Failed to delete Borrow")?;

    transaction
        .commit()
        .await
        .http_internal_error("Failed to commit")?;

    Ok(())
}
