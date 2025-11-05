use axum::extract::ws::Message;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

// Maximum messages per room (enforced by database trigger)
#[allow(dead_code)]
const MAX_MESSAGE_HISTORY: usize = 1000;

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Postgres>,
    pub rooms: Arc<RwLock<HashMap<String, RoomState>>>,
}

pub struct RoomState {
    pub connections: HashMap<String, mpsc::UnboundedSender<Message>>,
    pub usernames: HashMap<String, String>,
}

impl RoomState {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            usernames: HashMap::new(),
        }
    }

    pub fn online_count(&self) -> usize {
        self.connections.len()
    }

    pub fn add_connection(&mut self, user_id: String, username: String, tx: mpsc::UnboundedSender<Message>) {
        self.connections.insert(user_id.clone(), tx);
        self.usernames.insert(user_id, username);
    }

    pub fn remove_connection(&mut self, user_id: &str) -> Option<String> {
        self.connections.remove(user_id)?;
        self.usernames.remove(user_id)
    }

    pub fn get_username(&self, user_id: &str) -> Option<String> {
        self.usernames.get(user_id).cloned()
    }

    pub fn broadcast(&self, message: Message, exclude_user: Option<&str>) {
        for (user_id, tx) in &self.connections {
            if let Some(excluded) = exclude_user {
                if user_id == excluded {
                    continue;
                }
            }
            let _ = tx.send(message.clone());
        }
    }

    pub fn send_to_user(&self, user_id: &str, message: Message) {
        if let Some(tx) = self.connections.get(user_id) {
            let _ = tx.send(message);
        }
    }
}

impl AppState {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self {
            db,
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Clone for RoomState {
    fn clone(&self) -> Self {
        Self {
            connections: HashMap::new(),
            usernames: self.usernames.clone(),
        }
    }
}
