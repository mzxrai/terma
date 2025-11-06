use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::Response,
};
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use terma_shared::{ChatMessage, ClientMessage, ServerMessage};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::{db, state::AppState};

const MAX_MESSAGE_LENGTH: usize = 4096;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, room_id, state))
}

async fn handle_socket(socket: WebSocket, room_id: String, state: AppState) {
    // Verify room exists
    let room_exists = match db::room_exists(&state.db, &room_id).await {
        Ok(exists) => exists,
        Err(e) => {
            error!("Failed to check room existence: {}", e);
            return;
        }
    };

    if !room_exists {
        let _ = socket.close().await;
        return;
    }

    let (mut sender, mut receiver) = socket.split();

    // Create channel for this connection
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Wait for Join message
    let (user_id, username) = loop {
        match receiver.next().await {
            Some(Ok(Message::Text(text))) => {
                if let Ok(ClientMessage::Join { user_id, username }) =
                    ClientMessage::from_json(&text)
                {
                    break (user_id, username);
                }
            }
            Some(Ok(Message::Close(_))) | None => return,
            _ => continue,
        }
    };

    info!("User {} joined room {}", user_id, room_id);

    // Add connection to room
    let mut rooms = state.rooms.write().await;
    let room = rooms
        .entry(room_id.clone())
        .or_insert_with(crate::state::RoomState::new);

    room.add_connection(user_id.clone(), username.clone(), tx);

    // Get online count before releasing lock
    let online_count = room.online_count();

    drop(rooms);

    // Send welcome message with history from database
    let history = db::get_message_history(&state.db, &room_id)
        .await
        .unwrap_or_else(|e| {
            error!("Failed to load message history: {}", e);
            Vec::new()
        });

    let welcome = ServerMessage::Welcome {
        room_id: room_id.clone(),
        user_id: user_id.clone(),
        online_count,
    };

    if sender
        .send(Message::Text(welcome.to_json().unwrap()))
        .await
        .is_err()
    {
        return;
    }

    if !history.is_empty() {
        let history_msg = ServerMessage::History { messages: history };
        if sender
            .send(Message::Text(history_msg.to_json().unwrap()))
            .await
            .is_err()
        {
            return;
        }
    }

    // Broadcast user joined
    let rooms = state.rooms.read().await;
    if let Some(room) = rooms.get(&room_id) {
        let online_count = room.online_count();
        let joined_msg = ServerMessage::UserJoined {
            user_id: user_id.clone(),
            username: username.clone(),
            timestamp: Utc::now(),
            online_count,
        };
        room.broadcast(Message::Text(joined_msg.to_json().unwrap()), Some(&user_id));
    }
    drop(rooms);

    // Spawn task to send messages to client
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    let state_clone = state.clone();
    let room_id_clone = room_id.clone();
    let user_id_clone = user_id.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(client_msg) = ClientMessage::from_json(&text) {
                        handle_client_message(
                            client_msg,
                            &room_id_clone,
                            &user_id_clone,
                            &state_clone,
                        )
                        .await;
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    // User disconnected - clean up
    info!("User {} left room {}", user_id, room_id);

    let mut rooms = state.rooms.write().await;
    if let Some(room) = rooms.get_mut(&room_id) {
        if let Some(username) = room.remove_connection(&user_id) {
            let online_count = room.online_count();

            let left_msg = ServerMessage::UserLeft {
                user_id: user_id.clone(),
                username,
                timestamp: Utc::now(),
                online_count,
            };

            room.broadcast(Message::Text(left_msg.to_json().unwrap()), None);
        }

        // Clean up empty rooms
        if room.connections.is_empty() {
            rooms.remove(&room_id);
        }
    }
}

async fn handle_client_message(msg: ClientMessage, room_id: &str, user_id: &str, state: &AppState) {
    match msg {
        ClientMessage::SendMessage { content } => {
            if content.trim().is_empty() {
                return;
            }

            // Validate message length
            if content.len() > MAX_MESSAGE_LENGTH {
                let rooms = state.rooms.read().await;
                if let Some(room) = rooms.get(room_id) {
                    let error_msg = ServerMessage::Error {
                        message: format!(
                            "Message too long. Maximum length is {} characters.",
                            MAX_MESSAGE_LENGTH
                        ),
                    };
                    room.send_to_user(user_id, Message::Text(error_msg.to_json().unwrap()));
                }
                return;
            }

            let rooms = state.rooms.read().await;
            if let Some(room) = rooms.get(room_id) {
                let username = room
                    .get_username(user_id)
                    .unwrap_or_else(|| "Unknown".to_string());

                let chat_msg =
                    ChatMessage::new(room_id.to_string(), user_id.to_string(), username, content);

                // Save message to database
                if let Err(e) = db::save_message(&state.db, &chat_msg).await {
                    error!("Failed to save message to database: {}", e);
                    // Continue broadcasting even if save fails
                }

                let server_msg = ServerMessage::Message { message: chat_msg };

                room.broadcast(Message::Text(server_msg.to_json().unwrap()), None);
            }
        }
        ClientMessage::Ping => {
            let rooms = state.rooms.read().await;
            if let Some(room) = rooms.get(room_id) {
                let pong = ServerMessage::Pong;
                room.send_to_user(user_id, Message::Text(pong.to_json().unwrap()));
            }
        }
        ClientMessage::Join { .. } => {
            warn!("Received Join message after connection established");
        }
    }
}
