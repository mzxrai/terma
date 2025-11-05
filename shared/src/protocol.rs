use crate::models::ChatMessage;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Join { user_id: String },
    SendMessage { content: String },
    Ping,
}

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Welcome {
        room_id: String,
        user_id: String,
        online_count: usize,
    },
    History {
        messages: Vec<ChatMessage>,
    },
    Message {
        message: ChatMessage,
    },
    UserJoined {
        user_id: String,
        timestamp: DateTime<Utc>,
        online_count: usize,
    },
    UserLeft {
        user_id: String,
        timestamp: DateTime<Utc>,
        online_count: usize,
    },
    Error {
        message: String,
    },
    Pong,
}

impl ClientMessage {
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json(s: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}

impl ServerMessage {
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json(s: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}
