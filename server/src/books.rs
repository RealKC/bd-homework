use axum::{extract::State, Json};
use schema::books::{Author, Book};
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
