# Terma

A real-time chat application that lives in your terminal.

## Overview

Terma is a minimalist chat application built for the terminal. Create a chat room from a web interface, share a one-line install command, and anyone can join the conversation from their command line. No registration, no authentication, just instant communication.

## Try It

Visit [terma.mattmay.dev](https://terma.mattmay.dev) to create a room and get a one-line install command.

## Features

- **Terminal-based UI**: Clean interface built with Ratatui
- **Real-time messaging**: WebSocket-powered bidirectional communication
- **Persistent history**: PostgreSQL storage with automatic 1000-message limit per room
- **One-line installation**: curl-based installer downloads and runs the client binary
- **Cross-platform**: macOS and Linux support (x86_64 and ARM64)
- **Text selection**: Click and drag to select, right-click to copy
- **Clipboard integration**: OSC-52 terminal escape sequences work over SSH
- **Native notifications**: macOS system notifications for incoming messages
- **Presence indicators**: Join/leave events with online user counts

## Architecture

Terma is built as a Cargo workspace with three crates:

- **`shared/`**: Common protocol and data models used by both client and server
- **`server/`**: Axum web server with WebSocket handlers and PostgreSQL persistence
- **`client/`**: Ratatui terminal UI with async WebSocket client

### Technology Stack

**Server:**
- Axum for HTTP and WebSocket handling
- SQLx for PostgreSQL with compile-time query verification
- Tokio async runtime

**Client:**
- Ratatui for terminal UI rendering
- tui-textarea for multi-line text input
- tokio-tungstenite for WebSocket client
- crossterm for terminal manipulation

## Using Terma

### Creating and Joining Rooms

1. Visit the web interface to create a room
2. Copy the one-line install command
3. Share it with others
4. Run the command to download and launch the client

### Terminal Controls

- **Enter**: Send message
- **Shift+Enter**: Insert newline
- **Alt+↑/↓**: Scroll up/down
- **Mouse scroll**: Scroll through history
- **Click and drag**: Select text
- **Right-click**: Copy selected text
- **Ctrl+C**: Quit (or copy selected text if active)

## Project Structure

```
terma/
├── shared/           # Shared protocol library
│   └── src/
│       ├── models.rs     # Data structures (Room, ChatMessage, User)
│       └── protocol.rs   # WebSocket message types
├── server/           # Axum web server
│   ├── migrations/       # SQLx database migrations
│   └── src/
│       ├── db.rs         # Database operations
│       ├── state.rs      # Application state and room registry
│       ├── ws.rs         # WebSocket handler
│       └── handlers/     # HTTP routes
└── client/           # Ratatui terminal client
    └── src/
        ├── app.rs        # Application state
        ├── ui.rs         # UI rendering
        ├── events.rs     # Keyboard and mouse input
        ├── connection.rs # WebSocket client
        ├── clipboard.rs  # OSC-52 clipboard integration
        └── notifications.rs # macOS native notifications
```

## How It Works

### Connection Flow

1. Client connects to server via WebSocket at `/ws/<room-id>`
2. Server validates room existence and sends welcome message
3. Server sends message history (last 1000 messages)
4. Client and server exchange messages in real-time
5. Server broadcasts messages to all connected clients in the room

### Message Protocol

All WebSocket messages use JSON with a `type` field:

**Client → Server:**
- `Join`: Initial connection with user_id
- `SendMessage`: Send a chat message
- `Ping`: Keep-alive ping

**Server → Client:**
- `Welcome`: Connection confirmation with online user count
- `History`: Recent message history
- `Message`: New chat message from another user
- `UserJoined`: User joined notification
- `UserLeft`: User left notification
- `Error`: Error message
- `Pong`: Ping response

### Data Persistence

Rooms and messages are stored in PostgreSQL:

- Each room stores up to 1000 messages
- A database trigger automatically removes oldest messages when limit is exceeded
- Messages include timestamps, user info, and content
- Indexes optimize message retrieval by room and timestamp

## Implementation Details

### Custom Base64 Encoder

The clipboard module implements a custom base64 encoder for OSC-52 escape sequences, enabling clipboard support across SSH sessions without external dependencies.

### Database Trigger for Message Limits

PostgreSQL triggers automatically delete old messages when the 1000-message limit is exceeded, guaranteeing consistency even with multiple server instances.

### Render Cache System

The client maintains a render cache for text selection, mapping mouse coordinates to text positions across word-wrapped content.

### Cross-Platform Binary Distribution

GitHub Actions builds binaries for macOS (x86_64, ARM64) and Linux (x86_64, ARM64) using musl for static linking. The install script detects OS and architecture to download the appropriate binary.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Author

Built by Matt May - [GitHub](https://github.com/premshree) - [Website](https://mattmay.dev)
