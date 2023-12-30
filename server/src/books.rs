use axum::{extract::State, Json};
use schema::books::{Author, Book, BorrowReply, BorrowRequest};
use sqlx::SqlitePool;

use crate::error::{IntoRouteError, RouteError};

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

        sqlx::query!(
            r#"
INSERT INTO Borrows(book_id, user_id) VALUES (?, ?);
        "#,
            request.book_id,
            request.cookie.id
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
