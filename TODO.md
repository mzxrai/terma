# Terma Recovery TODO

This document tracks work lost in the recent directory restore. These features were implemented but need to be re-implemented.

## Username Feature ✅ COMPLETED

### Shared Protocol (`shared/src/`)
- [x] Add `username: String` field to `ClientMessage::Join` in protocol.rs
- [x] Add `username: String` field to `ServerMessage::UserJoined` in protocol.rs
- [x] Add `username: String` field to `ServerMessage::UserLeft` in protocol.rs
- [x] Add `username: String` field to `ChatMessage` struct in models.rs
- [x] Update `ChatMessage::new()` to accept username parameter (4 args total)

### Client Config System (`client/src/`)
- [x] Create `config.rs` module with Config struct using JSON format
- [x] Store config in `~/.terma/config.json` (not `~/.terma_username`)
- [x] Config struct should have: `username: String` and `host: Option<String>`
- [x] Implement `get_or_prompt_username()` - prompts once, saves to config
- [x] Implement `save_host()` - saves host to config for future use
- [x] Implement `get_host()` - retrieves saved host from config

### Client App Updates (`client/src/app.rs`)
- [x] Add `username: String` field to `App` struct
- [x] Update `App::new()` to accept username parameter
- [x] Change `DisplayMessage` to store `username: String` instead of `user_id`
- [x] Add `is_own_message: bool` field to `DisplayMessage`
- [x] Update `add_chat_message()` to set `is_own_message` flag
- [x] Update `format_for_display()` to show username instead of user_id

### Client Main (`client/src/main.rs`)
- [x] Add `mod config;` declaration
- [x] Call `config::get_or_prompt_username()` before TUI starts
- [x] Pass username to `App::new()`
- [x] Pass username to `connection::Connection::connect()`
- [x] Update message handlers to display usernames in join/leave events

### Client Connection (`client/src/connection.rs`)
- [x] Add `username: String` parameter to `connect()` function
- [x] Pass username in Join message

### Client UI (`client/src/ui.rs`)
- [x] Update header to show `"You: {username}"` instead of `"You: {user_id}"`
- [x] Update message rendering to use `is_own_message` flag for highlighting

### Server State (`server/src/state.rs`)
- [x] Add `usernames: HashMap<String, String>` field to `RoomState`
- [x] Update `RoomState::new()` to initialize usernames HashMap
- [x] Update `add_connection()` to accept and store username (3 params total)
- [x] Update `remove_connection()` to return `Option<String>` (the username)
- [x] Add `get_username()` method to retrieve username by user_id
- [x] Update `Clone` impl to include usernames field

### Server WebSocket (`server/src/ws.rs`)
- [x] Extract username from Join message (pattern match both user_id and username)
- [x] Store username when calling `add_connection()`
- [x] Include username in UserJoined broadcast message
- [x] Retrieve and include username in UserLeft broadcast message
- [x] Get username from room state when creating ChatMessage in SendMessage handler

## Message Limits ✅ COMPLETED

### Server State (`server/src/state.rs`)
- [x] Change `MAX_MESSAGE_HISTORY` from 100 to 500 (already completed in previous commit)

### Server WebSocket (`server/src/ws.rs`)
- [x] Add `const MAX_MESSAGE_LENGTH: usize = 4096;` (already defined)
- [x] Validate message length in SendMessage handler
- [x] Send Error message if content exceeds 4096 characters

## Shorthand Command Support ✅ COMPLETED

### Client Config (`client/src/config.rs`)
- [x] Remove host storage functions (not needed - host is compile-time only)
- [x] Keep only username storage functionality

### Client Main (`client/src/main.rs`)
- [x] Support `terma <room_id>` (uses compile-time DEFAULT_HOST)
- [x] Support `terma <host> <room_id>` (allows override)
- [x] Use TERMA_DEFAULT_HOST environment variable at compile time
- [x] Default to "localhost:3000" for development builds
- [x] Show helpful usage message with examples

## Testing Checklist

After implementation:
- [x] Test username prompt on first run (completed previously)
- [x] Test username persists in `~/.terma/config.json` (completed previously)
- [x] Test username displays correctly in chat messages (completed previously)
- [x] Test username displays in header ("You: Matt") (completed previously)
- [x] Test join/leave events show usernames (completed previously)
- [ ] Test message limit of 4096 characters (needs testing)
- [x] Test 500 message history (already set)
- [x] Test multiple users with same username can join (no uniqueness check)
- [x] Test user can open multiple terminal sessions with same username
- [ ] Test shorthand command `terma <room_id>` (needs testing)
- [ ] Test full command `terma <host> <room_id>` (needs testing)

## Notes

- Username uniqueness is **NOT** enforced - same person can have multiple sessions
- Config is JSON format for future extensibility
- Message length validation happens server-side
- Client should handle Error messages from server gracefully
- Host is compile-time only (via TERMA_DEFAULT_HOST env var)
- Development builds default to "localhost:3000"
- Production builds should set TERMA_DEFAULT_HOST during GitHub Actions build
