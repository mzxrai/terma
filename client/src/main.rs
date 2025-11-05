mod app;
mod config;
mod connection;
mod events;
mod ui;

use anyhow::{Context, Result};
use app::App;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::time::Duration;
use terma_shared::{ClientMessage, ServerMessage};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Get default host from compile-time environment variable or use localhost:3000
    const DEFAULT_HOST: &str = match option_env!("TERMA_DEFAULT_HOST") {
        Some(host) => host,
        None => "localhost:3000",
    };

    let (host, room_id) = match args.len() {
        2 => {
            // Shorthand: terma <room_id>
            (DEFAULT_HOST.to_string(), args[1].clone())
        }
        3 => {
            // Full: terma <host> <room_id>
            (args[1].clone(), args[2].clone())
        }
        _ => {
            eprintln!("Usage: {} <room_id>", args[0]);
            eprintln!("   or: {} <host> <room_id>", args[0]);
            eprintln!();
            eprintln!("Examples:");
            eprintln!("  {} abc123", args[0]);
            eprintln!("  {} localhost:3000 abc123", args[0]);
            std::process::exit(1);
        }
    };

    // Get or prompt for username
    let username = config::get_or_prompt_username()
        .context("Failed to get username")?;

    // Generate a random user ID
    let user_id = Uuid::new_v4().to_string()[..8].to_string();

    // Connect to server
    let (conn, mut rx) = connection::Connection::connect(&host, &room_id, user_id.clone(), username.clone())
        .await
        .context("Failed to establish connection")?;

    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode. Make sure you're running in a terminal (not via pipe or redirect).")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.show_cursor()?;

    // Create app
    let mut app = App::new(room_id, user_id, username);

    // Run app
    let result = run_app(&mut terminal, &mut app, &conn, &mut rx).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    conn: &connection::Connection,
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<ServerMessage>,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|f| ui::render(f, app))?;

        // Check for quit
        if app.should_quit {
            break;
        }

        // Check for keyboard events (non-blocking with short timeout)
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if let Some(message) = events::handle_key_event(app, key) {
                    conn.send(ClientMessage::SendMessage { content: message })?;
                }
            }
        }

        // Check for incoming WebSocket messages (non-blocking)
        match rx.try_recv() {
            Ok(msg) => {
                handle_server_message(app, msg);
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                // No message, continue
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                // Connection closed
                break;
            }
        }

        // Small yield to prevent CPU spinning
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    Ok(())
}

fn handle_server_message(app: &mut App, msg: ServerMessage) {
    match msg {
        ServerMessage::Welcome {
            online_count,
            ..
        } => {
            app.connected = true;
            app.online_count = online_count;
            app.add_system_message(format!(
                "Connected to room {}. {} user(s) online.",
                app.room_id, online_count
            ));
        }
        ServerMessage::History { messages } => {
            for msg in messages {
                app.add_chat_message(msg);
            }
        }
        ServerMessage::Message { message } => {
            app.add_chat_message(message);
        }
        ServerMessage::UserJoined {
            user_id,
            username,
            online_count,
            ..
        } => {
            app.online_count = online_count;
            if user_id != app.user_id {
                app.add_system_message(format!("{} joined. {} user(s) online.", username, online_count));
            }
        }
        ServerMessage::UserLeft {
            username,
            online_count,
            ..
        } => {
            app.online_count = online_count;
            app.add_system_message(format!("{} left. {} user(s) online.", username, online_count));
        }
        ServerMessage::Error { message } => {
            app.add_system_message(format!("Error: {}", message));
        }
        ServerMessage::Pong => {}
    }
}
