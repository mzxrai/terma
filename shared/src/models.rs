use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub room_id: String,
    pub user_id: String,
    pub username: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub connected_at: DateTime<Utc>,
}

impl ChatMessage {
    pub fn new(room_id: String, user_id: String, username: String, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            room_id,
            user_id,
            username,
            content,
            timestamp: Utc::now(),
        }
    }
}

impl Room {
    pub fn new(id: String) -> Self {
        Self {
            id,
            created_at: Utc::now(),
        }
    }
}
