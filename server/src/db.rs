use anyhow::Result;
use chrono::Utc;
use sqlx::{Pool, Postgres, PgPool};
use terma_shared::{ChatMessage, Room};
use uuid::Uuid;

pub async fn init_db(database_url: &str) -> Result<Pool<Postgres>> {
    let pool = PgPool::connect(database_url).await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}

pub async fn create_room(pool: &Pool<Postgres>, room_id: String) -> Result<Room> {
    let created_at = Utc::now();

    sqlx::query("INSERT INTO rooms (id, created_at) VALUES ($1, $2)")
        .bind(&room_id)
        .bind(created_at)
        .execute(pool)
        .await?;

    Ok(Room {
        id: room_id,
        created_at,
    })
}

pub async fn room_exists(pool: &Pool<Postgres>, room_id: &str) -> Result<bool> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM rooms WHERE id = $1"
    )
    .bind(room_id)
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
}

pub async fn save_message(pool: &Pool<Postgres>, msg: &ChatMessage) -> Result<()> {
    sqlx::query(
        "INSERT INTO messages (room_id, user_id, username, content, timestamp)
         VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(&msg.room_id)
    .bind(&msg.user_id)
    .bind(&msg.username)
    .bind(&msg.content)
    .bind(msg.timestamp)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_message_history(pool: &Pool<Postgres>, room_id: &str) -> Result<Vec<ChatMessage>> {
    let messages = sqlx::query_as::<_, (String, String, String, String, chrono::DateTime<Utc>)>(
        "SELECT room_id, user_id, username, content, timestamp
         FROM messages
         WHERE room_id = $1
         ORDER BY timestamp ASC
         LIMIT 1000"
    )
    .bind(room_id)
    .fetch_all(pool)
    .await?;

    Ok(messages
        .into_iter()
        .map(|(room_id, user_id, username, content, timestamp)| ChatMessage {
            id: Uuid::new_v4(),
            room_id,
            user_id,
            username,
            content,
            timestamp,
        })
        .collect())
}
