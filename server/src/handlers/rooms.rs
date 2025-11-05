use axum::{extract::State, http::StatusCode, Json};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use crate::{db, state::AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoomResponse {
    pub room_id: String,
    pub install_command: String,
}

pub async fn create_room(
    State(state): State<AppState>,
) -> Result<Json<CreateRoomResponse>, StatusCode> {
    let room_id = nanoid!(10);

    db::create_room(&state.db, room_id.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let host = std::env::var("HOST").unwrap_or_else(|_| "localhost:3000".to_string());
    let install_command = format!(r#"sh -c "$(curl -fsSL http://{}/join/{})""#, host, room_id);

    Ok(Json(CreateRoomResponse {
        room_id,
        install_command,
    }))
}
