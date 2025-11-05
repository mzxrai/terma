use anyhow::Result;
use chrono::Utc;
use sqlx::{Pool, Postgres, PgPool};
use terma_shared::Room;

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
