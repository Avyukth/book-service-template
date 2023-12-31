use anyhow::Result;
// use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, SqlitePool};
use tokio::sync::{Mutex, RwLock};
// use utoipa::{OpenApi};

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub author: String,
}

struct BookCache {
    all_books: RwLock<Option<Vec<Book>>>,
}

pub async fn init_db() -> Result<SqlitePool> {
    let database_url = std::env::var("DATABASE_URL")?;
    // .expect("DATABASE_URL must be present in environment variables");
    let connection_pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!().run(&connection_pool).await?;
    Ok(connection_pool)
}

pub async fn get_all_books(connection_pool: &SqlitePool) -> Result<Vec<Book>> {
    Ok(
        sqlx::query_as::<_, Book>("SELECT * FROM books order by title, author")
            .fetch_all(connection_pool)
            .await?,
    )
}

pub async fn get_book_by_id(connection_pool: &SqlitePool, id: i32) -> Result<Book> {
    Ok(
        sqlx::query_as::<_, Book>("SELECT * FROM books where id = $1")
            .bind(id)
            .fetch_one(connection_pool)
            .await?,
    )
}

pub async fn add_book<S: ToString>(
    connection_pool: &SqlitePool,
    title: S,
    author: S,
) -> Result<i32> {
    let title = title.to_string();
    let author = author.to_string();
    Ok(
        sqlx::query("INSERT INTO books (title, author) VALUES ($1, $2) RETURNING id")
            .bind(title)
            .bind(author)
            .fetch_one(connection_pool)
            .await?
            .get(0),
    )
}

pub async fn update_book(connection_pool: &SqlitePool, book: &Book) -> Result<()> {
    sqlx::query("UPDATE books SET title = $1, author = $2 WHERE id = $3")
        .bind(&book.title)
        .bind(&book.author)
        .bind(&book.id)
        .execute(connection_pool)
        .await?;
    Ok(())
}

pub async fn delete_book(connection_pool: &SqlitePool, id: i32) -> Result<()> {
    sqlx::query("DELETE FROM books WHERE id = $1")
        .bind(id)
        .execute(connection_pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    #[sqlx::test]
    async fn test_get_all_books() {
        dotenv::dotenv().ok();
        let connection_pool = init_db().await.unwrap();
        let books = get_all_books(&connection_pool).await.unwrap();
        assert!(!books.is_empty());
    }

    #[sqlx::test]
    async fn test_get_book_by_id() {
        dotenv::dotenv().ok();
        let connection_pool = init_db().await.unwrap();
        let book = get_book_by_id(&connection_pool, 1).await.unwrap();
        assert_eq!(book.id, 1);
        assert_eq!("Hands-on Rust", book.title);
        assert_eq!("Wolverson, Herbert", book.author);
    }

    #[sqlx::test]
    async fn test_create() {
        dotenv::dotenv().ok();
        let cnn = init_db().await.unwrap();
        let new_id = add_book(&cnn, "Test Book", "Test Author").await.unwrap();
        let new_book = get_book_by_id(&cnn, new_id).await.unwrap();
        assert_eq!(new_id, new_book.id);
        assert_eq!("Test Book", new_book.title);
        assert_eq!("Test Author", new_book.author);
    }

    #[sqlx::test]
    async fn test_update() {
        dotenv::dotenv().ok();
        let cnn = init_db().await.unwrap();
        let mut book = get_book_by_id(&cnn, 2).await.unwrap();
        book.title = "Updated Book".to_string();
        update_book(&cnn, &book).await.unwrap();
        let updated_book = get_book_by_id(&cnn, 2).await.unwrap();
        assert_eq!("Updated Book", updated_book.title);
    }

    #[sqlx::test]
    async fn test_delete() {
        dotenv::dotenv().ok();
        let cnn = init_db().await.unwrap();
        let new_id = add_book(&cnn, "DeleteMe", "Test Author").await.unwrap();
        let _new_book = get_book_by_id(&cnn, new_id).await.unwrap();
        delete_book(&cnn, new_id).await.unwrap();
        let all_books = get_all_books(&cnn).await.unwrap();
        assert!(all_books.iter().find(|b| b.title == "DeleteMe").is_none());
    }
}
