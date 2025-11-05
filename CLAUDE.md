# Terma Development Guidelines

## Project Overview
Terma is a real-time terminal chat application built with Rust, Ratatui, and Axum. It allows users to create chat rooms via a web interface and connect to them through a terminal client.

## Architecture
- **Cargo Workspace** with 3 crates: `server`, `client`, `shared`
- **Server (Axum)**: Web UI + WebSocket server + SQLite for persistent rooms
- **Client (Ratatui)**: Terminal UI connecting via WebSocket
- **Shared**: Common protocol types and message structures

## Development Principles

### Code Organization
- Keep files small and manageable
- Keep code modular - don't write massive single files
- Break functionality apart into logical modules
- Use modern Rust development principles

### Quality Standards
- **Zero warnings policy**: All code must compile without warnings
- **Test before completion**: Test your work before declaring it done - do not assume
- **No shortcuts permitted**: Make this properly, following best practices

### Dependencies
- **Database**: Use SQLite (no Postgres)
- **Infrastructure**: Use the simplest possible thing that will work
- **Rust & Ratatui**: Required for terminal portion
- **Rust & Axum**: Preferred for webapp/server

## Key Features
- Persistent rooms (survive server restart)
- Real-time messaging via WebSocket
- Presence indicators (join/leave events)
- Limited message history (last 100 messages in memory)
- Anonymous access (link-based, no authentication)
- One-liner installation via curl script
- Cross-NAT functionality (works over internet, not just local network)

## Project Structure
```
terma/
├── Cargo.toml              # Workspace configuration
├── shared/                 # Shared protocol library
│   └── src/
│       ├── lib.rs
│       ├── models.rs       # Data structures (Room, ChatMessage, User)
│       └── protocol.rs     # WebSocket message types
├── server/                 # Axum web server
│   ├── migrations/         # SQLx database migrations
│   └── src/
│       ├── main.rs         # Server entry point
│       ├── db.rs           # Database operations
│       ├── state.rs        # Application state management
│       ├── ws.rs           # WebSocket handler
│       └── handlers/       # HTTP route handlers
│           ├── mod.rs
│           ├── rooms.rs    # Room creation API
│           ├── web.rs      # Web interface
│           └── install.rs  # Install script
└── client/                 # Ratatui terminal client
    └── src/
        ├── main.rs         # Client entry point
        ├── app.rs          # Application state
        ├── ui.rs           # UI rendering with Ratatui
        ├── events.rs       # Keyboard event handling
        └── connection.rs   # WebSocket client connection
```

## Running the Application

### Start the Server
```bash
cargo run --release --bin terma-server
```

The server will:
- Initialize SQLite database at `terma.db`
- Start web server on `http://localhost:3000`
- Serve the web UI for creating rooms
- Handle WebSocket connections for chat

### Environment Variables
- `DATABASE_URL`: SQLite connection string (default: `sqlite:terma.db`)
- `BIND_ADDR`: Server bind address (default: `0.0.0.0:3000`)
- `HOST`: Public hostname for install commands (default: `localhost:3000`)

### Connect a Client
```bash
cargo run --release --bin terma-client -- localhost:3000 <room_id>
```

## Installation Flow
1. User visits web interface at http://localhost:3000
2. Clicks "Create New Room" to generate a room
3. Gets a shareable link and curl one-liner command
4. Shares command with others: `curl -sSL <url>/install.sh | sh -s -- <room_id>`
5. Script downloads the client binary and starts chat session
6. All participants can chat in real-time with presence indicators

## Protocol
WebSocket messages use JSON with a `type` field:

### Client → Server
- `Join`: Initial connection with user_id
- `SendMessage`: Send chat message
- `Ping`: Keep-alive ping

### Server → Client
- `Welcome`: Connection confirmation with online count
- `History`: Recent messages (up to 100)
- `Message`: New chat message
- `UserJoined`: User joined notification
- `UserLeft`: User left notification
- `Error`: Error message
- `Pong`: Ping response

## Database Schema
```sql
CREATE TABLE rooms (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL
);
```

Rooms are persistent across server restarts, but messages are ephemeral (kept in memory only, last 100 messages per room).
