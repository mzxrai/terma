# Terma

A real-time terminal chat application. Create chat rooms via web interface, connect through terminal clients.

## Features

- **One-liner installation** - Install and connect with a single curl command
- **Real-time messaging** - WebSocket-based instant communication
- **Persistent rooms** - Rooms survive server restarts (PostgreSQL backend)
- **Anonymous access** - No authentication required, just share a room link
- **Cross-platform** - Works on macOS (x86_64, ARM64) and Linux (x86_64, ARM64)
- **Message history** - Last 500 messages stored per room
- **Presence indicators** - See when users join/leave and online count

## Quick Start

### Installation & Usage

1. Visit your Terma server's web interface (e.g., `https://your-server.com`)
2. Click "Create New Room"
3. Share the one-liner command with others:

```bash
curl -sSL https://your-server.com/join/ROOM_ID | sh
```

The script will:
- Download the terma client binary
- Install it to `~/.local/bin/terma`
- Automatically connect you to the room

### Running the Client Later

After installation, you can connect to any room using:

```bash
terma ROOM_ID
```

Or specify a custom server:

```bash
terma your-server.com:443 ROOM_ID
```

## Architecture

Terma is built with Rust using:
- **Server**: Axum web framework + WebSocket
- **Client**: Ratatui terminal UI
- **Database**: PostgreSQL (or any SQLx-compatible database)
- **Protocol**: JSON over WebSocket

### Project Structure

```
terma/
├── server/          # Axum web server + WebSocket handler
├── client/          # Ratatui terminal client
└── shared/          # Shared protocol and data structures
```

## Development

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- PostgreSQL database (we recommend [Neon](https://neon.tech/) for free hosting)

### Setup

1. Clone the repository:
```bash
git clone https://github.com/mzxrai/terma.git
cd terma
```

2. Set up environment variables:
```bash
cp .env.example .env
# Edit .env with your database connection string
```

3. Run migrations:
```bash
cd server && cargo sqlx migrate run
```

4. Build the project:
```bash
cargo build --release
```

### Running Locally

**Start the server:**
```bash
cargo run --release --bin terma-server
```

Server will start on `http://localhost:3000`

**Connect a client:**
```bash
cargo run --release -p terma-client ROOM_ID
```

## Deployment

### Environment Variables

#### Build-time (GitHub Actions)
- `TERMA_DEFAULT_HOST` - Default server hostname compiled into client binary
  - Example: `terma.example.com:443`
  - Omit for development builds (defaults to `localhost:3000`)

#### Runtime (Server)
- `DATABASE_URL` - PostgreSQL connection string (required)
- `BIND_ADDR` - Server bind address (default: `0.0.0.0:3000`)
- `HOST` - Public hostname for install commands (default: `localhost:3000`)
- `GITHUB_REPO` - GitHub repo for binary downloads (optional, format: `owner/repo`)

### Deploying to Cloudflare Containers

Terma works great on Cloudflare Containers with a Neon PostgreSQL database:

1. Set up a Neon database at [neon.tech](https://neon.tech/)
2. From `cloudflare/worker/`, install Worker dependencies (`npm install`) so Wrangler can bundle the Worker and `@cloudflare/containers` helpers.
3. Configure runtime secrets (`DATABASE_URL`, optional `HOST` override) via `npx wrangler secret put` (secrets are required because the Worker passes them into the container via the Container class). By default the Worker will serve from `terma-worker.mzxrai.workers.dev`.
4. Run `npx wrangler deploy` at the repo root; Wrangler will build the Docker image defined in `./Dockerfile`, push it to Cloudflare’s registry, and deploy the Worker that proxies all HTTP/WebSocket traffic to the singleton container.
5. The filesystem is ephemeral, but PostgreSQL provides persistence

### GitHub Actions Setup

To configure the default host for production builds:

1. Go to your GitHub repository settings
2. Navigate to **Settings** > **Secrets and variables** > **Actions** > **Variables**
3. Add a repository variable:
   - Name: `TERMA_DEFAULT_HOST`
   - Value: `your-server.com:443` (your production hostname)

Binaries will be automatically built and released to GitHub Releases on every push to main.

## Protocol

Messages are JSON over WebSocket:

### Client → Server
- `Join`: Initial connection with user_id and username
- `SendMessage`: Send a chat message
- `Ping`: Keep-alive ping

### Server → Client
- `Welcome`: Connection confirmation with online count
- `History`: Recent messages (up to 500)
- `Message`: New chat message
- `UserJoined`: User joined notification
- `UserLeft`: User left notification
- `Error`: Error message
- `Pong`: Ping response

## Configuration

### Message Limits
- **Message history**: 500 messages per room (in-memory)
- **Message length**: 4096 characters maximum

### Username
On first run, the client prompts for a username, which is stored in `~/.terma/config.json`

## Contributing

Contributions are welcome! Please ensure:
- Code compiles without warnings (`cargo build --release`)
- All tests pass
- Follow the existing code style

## License

[Add your license here]

## Credits

Built with ❤️ using Rust
