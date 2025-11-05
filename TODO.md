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

## Message Limits

### Server State (`server/src/state.rs`)
- [ ] Change `MAX_MESSAGE_HISTORY` from 100 to 500

### Server WebSocket (`server/src/ws.rs`)
- [ ] Add `const MAX_MESSAGE_LENGTH: usize = 4096;`
- [ ] Validate message length in SendMessage handler
- [ ] Send Error message if content exceeds 4096 characters

## Shorthand Command Support (Optional - Was in Progress)

This was partially implemented when the restore happened. May want to complete later:

### Client Config (`client/src/config.rs`)
- [ ] Ensure host storage functions are working

### Client Main (`client/src/main.rs`)
- [ ] Support `terma <room_id>` (reads host from config)
- [ ] Support `terma <host> <room_id>` (saves host to config)
- [ ] Show helpful error if host not configured

## Testing Checklist

After implementation:
- [ ] Test username prompt on first run
- [ ] Test username persists in `~/.terma/config.json`
- [ ] Test username displays correctly in chat messages
- [ ] Test username displays in header ("You: Matt")
- [ ] Test join/leave events show usernames
- [ ] Test message limit of 4096 characters
- [ ] Test 500 message history
- [ ] Test multiple users with same username can join (no uniqueness check)
- [ ] Test user can open multiple terminal sessions with same username

## Notes

- Username uniqueness is **NOT** enforced - same person can have multiple sessions
- Config is JSON format for future extensibility
- Message length validation happens server-side
- Client should handle Error messages from server gracefully
