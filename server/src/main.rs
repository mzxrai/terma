mod db;
mod handlers;
mod state;
mod ws;

use axum::{
    routing::{get, post},
    Router,
};
use state::AppState;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "terma_server=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize database
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:terma.db".to_string());

    let db = db::init_db(&database_url).await?;
    info!("Database initialized");

    // Create app state
    let state = AppState::new(db);

    // Build router
    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/api/rooms", post(handlers::create_room))
        .route("/join/:room_id", get(handlers::install_script))
        .route("/download/:filename", get(handlers::download_binary))
        .route("/ws/:room_id", get(ws::websocket_handler))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    info!("Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
