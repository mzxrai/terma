use anyhow::Result;
use chrono::Utc;
use sqlx::{Pool, Sqlite, SqlitePool};
use terma_shared::Room;

pub async fn init_db(database_url: &str) -> Result<Pool<Sqlite>> {
    let pool = SqlitePool::connect(database_url).await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}

pub async fn create_room(pool: &Pool<Sqlite>, room_id: String) -> Result<Room> {
    let created_at = Utc::now();
    let created_at_str = created_at.to_rfc3339();

    sqlx::query("INSERT INTO rooms (id, created_at) VALUES (?, ?)")
        .bind(&room_id)
        .bind(&created_at_str)
        .execute(pool)
        .await?;

    Ok(Room {
        id: room_id,
        created_at,
    })
}

pub async fn room_exists(pool: &Pool<Sqlite>, room_id: &str) -> Result<bool> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM rooms WHERE id = ?"
    )
    .bind(room_id)
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
}
