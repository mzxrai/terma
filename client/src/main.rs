mod app;
mod clipboard;
mod config;
mod connection;
mod events;
mod notifications;
mod ui;

use anyhow::{Context, Result};
use app::App;
use crossterm::{
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event, MouseButton, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
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
    let username = config::get_or_prompt_username().context("Failed to get username")?;

    // Generate a random user ID
    let user_id = Uuid::new_v4().to_string()[..8].to_string();

    // Connect to server
    let (conn, mut rx) =
        connection::Connection::connect(&host, &room_id, user_id.clone(), username.clone())
            .await
            .context("Failed to establish connection")?;

    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode. Make sure you're running in a terminal (not via pipe or redirect).")?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableBracketedPaste,
        EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.show_cursor()?;

    // Create app
    let mut app = App::new(room_id, user_id, username);

    // Run app
    let result = run_app(&mut terminal, &mut app, &conn, &mut rx).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableBracketedPaste,
        DisableMouseCapture
    )?;
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
        let mut did_work = false;

        // Draw UI
        terminal.draw(|f| ui::render(f, app))?;

        // Check for quit
        if app.should_quit {
            break;
        }

        // Check for keyboard and mouse events (non-blocking with short timeout)
        if event::poll(Duration::from_millis(50))? {
            did_work = true;
            match event::read()? {
                Event::Key(key) => {
                    if let Some(message) = events::handle_key_event(app, key) {
                        conn.send(ClientMessage::SendMessage { content: message })?;
                    }
                }
                Event::Mouse(mouse) => {
                    handle_mouse_event(app, mouse);
                }
                Event::Paste(text) => {
                    // Handle pasted text - insert it into the textarea without triggering sends
                    handle_paste_event(app, text);
                }
                _ => {}
            }
        }

        // Check for incoming WebSocket messages (non-blocking)
        loop {
            match rx.try_recv() {
                Ok(msg) => {
                    handle_server_message(app, msg);
                    did_work = true;
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                    break;
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    return Ok(());
                }
            }
        }

        if !did_work {
            // Small sleep when idle to avoid busy-looping
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    Ok(())
}

fn handle_mouse_event(app: &mut App, mouse: crossterm::event::MouseEvent) {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            // Scroll up in message history (increase offset to show older messages)
            app.scroll_up();
        }
        MouseEventKind::ScrollDown => {
            // Scroll down in message history (decrease offset to show newer messages)
            app.scroll_down();
        }
        MouseEventKind::Down(MouseButton::Left) => {
            if let Some(pos) = app.message_position_from_mouse(mouse.column, mouse.row) {
                app.start_selection(pos);
            } else {
                app.clear_selection();
            }
        }
        MouseEventKind::Drag(MouseButton::Left) | MouseEventKind::Up(MouseButton::Left) => {
            if let Some(pos) = app.message_position_from_mouse(mouse.column, mouse.row) {
                app.update_selection(pos);
            }
        }
        MouseEventKind::Up(MouseButton::Right) => {
            if app.has_selection() {
                if let Err(err) = app.copy_selection() {
                    app.add_system_message(format!("Copy failed: {}", err));
                }
            }
        }
        _ => {}
    }
}

fn handle_paste_event(app: &mut App, text: String) {
    // Insert pasted text into the textarea
    // TextArea will handle newlines properly without triggering sends
    app.input.insert_str(text);
}

fn handle_server_message(app: &mut App, msg: ServerMessage) {
    match msg {
        ServerMessage::Welcome { online_count, .. } => {
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
            // Send notification for messages from other users
            if message.user_id != app.user_id {
                notifications::send_notification(&message.username, &message.content);
            }
            app.add_chat_message(message);
        }
        ServerMessage::UserJoined {
            user_id,
            username,
            timestamp,
            online_count,
        } => {
            app.online_count = online_count;
            if user_id != app.user_id {
                app.add_system_message_with_time(
                    format!("{} joined. {} user(s) online.", username, online_count),
                    timestamp,
                );
            }
        }
        ServerMessage::UserLeft {
            username,
            timestamp,
            online_count,
            ..
        } => {
            app.online_count = online_count;
            app.add_system_message_with_time(
                format!("{} left. {} user(s) online.", username, online_count),
                timestamp,
            );
        }
        ServerMessage::Error { message } => {
            app.add_system_message(format!("Error: {}", message));
        }
        ServerMessage::Pong => {}
    }
}
