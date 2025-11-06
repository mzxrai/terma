# Terma

A real-time chat application that lives in your terminal.

## What is Terma?

Terma is a minimalist chat application built for the terminal. Create a chat room, share a link, and anyone can join the conversation from their command line. No registration, no authentication, just instant communication.

## Features

- **Terminal-based UI**: Clean, responsive interface built with Ratatui
- **Real-time messaging**: WebSocket-powered instant communication
- **Persistent history**: Messages are stored in PostgreSQL and survive server restarts
- **One-line installation**: Share a curl command to get others connected instantly
- **Cross-platform**: Works on macOS and Linux (x86_64 and ARM64)
- **Text selection and clipboard**: Native terminal clipboard support via OSC-52
- **Native notifications**: macOS system notifications for incoming messages
- **Presence indicators**: See when users join and leave rooms

## Quick Start

### Using a hosted instance

Visit [terma.mattmay.dev](https://terma.mattmay.dev) to create a room and get an install command.

### Running your own server

**Prerequisites:**
- Rust 1.75 or later
- PostgreSQL 14 or later
- Node.js (for development workflow)

**Setup:**

```bash
# Clone the repository
git clone https://github.com/yourusername/terma.git
cd terma

# Set up the database
createdb terma
export DATABASE_URL="postgres://localhost/terma"

# Run in development mode (auto-restart on changes)
npm run dev

# Or build for production
cargo build --release
cargo run --release --bin terma-server
```

The server will start on `http://localhost:3000`.

### Connecting as a client

```bash
# Build the client
cargo build --release --bin terma

# Connect to a room
./target/release/terma localhost:3000 <room-id>
```

Or use the one-line installer from the web interface.

## Architecture

Terma is built as a Cargo workspace with three crates:

- **`shared/`**: Common protocol and data models
- **`server/`**: Axum-based web server with WebSocket support
- **`client/`**: Ratatui-based terminal UI

### Technology Stack

**Server:**
- Axum for HTTP and WebSocket handling
- SQLx for PostgreSQL integration with compile-time query verification
- Tokio for async runtime

**Client:**
- Ratatui for terminal UI
- tui-textarea for multi-line text input
- tokio-tungstenite for WebSocket client
- crossterm for terminal manipulation

## Configuration

### Server Environment Variables

- `DATABASE_URL`: PostgreSQL connection string (default: `postgres://localhost/terma`)
- `BIND_ADDR`: Server bind address (default: `0.0.0.0:3000`)
- `HOST`: Public hostname for install commands (default: `localhost:3000`)

### Client Configuration

The client stores your username in `~/.terma/config.json`. You'll be prompted to set it on first run.

## Usage

### Creating a Room

1. Visit the web interface (e.g., `http://localhost:3000`)
2. Click "Create New Room"
3. Copy the install command or share the room link

### Chat Commands

- **Enter**: Send message
- **Shift+Enter**: Insert newline
- **Alt+↑/↓**: Scroll up/down
- **Mouse scroll**: Scroll through history
- **Click and drag**: Select text
- **Right-click**: Copy selected text
- **Ctrl+C**: Quit (or copy selected text if selection is active)

## Development

### Project Structure

```
terma/
├── shared/           # Shared protocol library
│   └── src/
│       ├── models.rs     # Data structures
│       └── protocol.rs   # WebSocket messages
├── server/           # Axum web server
│   ├── migrations/       # Database migrations
│   └── src/
│       ├── db.rs         # Database operations
│       ├── state.rs      # State management
│       ├── ws.rs         # WebSocket handler
│       └── handlers/     # HTTP routes
└── client/           # Ratatui terminal client
    └── src/
        ├── app.rs        # Application state
        ├── ui.rs         # UI rendering
        ├── events.rs     # Input handling
        └── connection.rs # WebSocket client
```

### Development Workflow

```bash
# Run both server and client watchers
npm run dev

# Server only (auto-restart on changes)
npm run dev:server

# Client only (auto-rebuild on changes)
npm run dev:client

# Run tests
cargo test

# Build everything
cargo build --release
```

### Database Migrations

```bash
# Run migrations
cargo sqlx migrate run

# Create a new migration
cargo sqlx migrate add <migration_name>

# Prepare for offline compilation
cargo sqlx prepare
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

## Building for Distribution

The project includes GitHub Actions workflows for building binaries:

```bash
# Build for specific platforms
cargo build --release --target x86_64-unknown-linux-musl
cargo build --release --target aarch64-unknown-linux-musl
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

Compiled binaries are suitable for distribution via the install script.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Author

Built by Matt May - [GitHub](https://github.com/premshree) - [Website](https://mattmay.dev)
