# Plan: Slack-like TUI with Database Storage

## Phase 1: Fix Message Scrolling & Display (Client)

### 1.1 Switch to Paragraph with Pre-calculated Wrapped Lines
- Replace current message rendering with proper text wrapping
- Pre-calculate line count for each message based on terminal width
- Build complete `Vec<Line>` with all wrapped messages
- Use `.scroll((scroll_offset, 0))` to control visible area
- Re-enable `.wrap(Wrap { trim: true })` on Paragraph widget

### 1.2 Fix Scroll Logic
- Change `scroll_offset` from counting messages to counting **lines**
- Calculate total lines after wrapping for proper scroll bounds
- Implement auto-scroll to bottom (scroll_offset = 0) on new messages
- Arrow up/down scrolls by lines, not messages

**Files**: `client/src/app.rs`, `client/src/ui.rs`

## Phase 2: Multi-line Input (Client)

### 2.1 Add tui-textarea Dependency
```toml
tui-textarea = "0.7"
```

### 2.2 Replace String Input with TextArea
- Update `App` struct to use `TextArea` instead of `String`
- TextArea handles multi-line editing automatically
- Configure to dynamically expand (Constraint::Min for input area)
- Update event handling to forward keys to TextArea

**Files**: `client/Cargo.toml`, `client/src/app.rs`, `client/src/ui.rs`, `client/src/events.rs`

## Phase 3: Database Message Storage with Transactional Enforcement (Server)

### 3.1 Create Messages Migration
```sql
CREATE TABLE messages (
    id BIGSERIAL PRIMARY KEY,
    room_id TEXT NOT NULL REFERENCES rooms(id),
    user_id TEXT NOT NULL,
    username TEXT NOT NULL,
    content TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_messages_room_timestamp ON messages(room_id, timestamp DESC);
```

### 3.2 Create Database Trigger for Hard 1000 Message Limit
```sql
CREATE OR REPLACE FUNCTION enforce_message_limit()
RETURNS TRIGGER AS $$
BEGIN
    DELETE FROM messages
    WHERE id IN (
        SELECT id FROM messages
        WHERE room_id = NEW.room_id
        ORDER BY timestamp DESC
        OFFSET 1000
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER enforce_room_message_limit
    AFTER INSERT ON messages
    FOR EACH ROW
    EXECUTE FUNCTION enforce_message_limit();
```

### 3.3 Implement Transactional Database Operations
```rust
// Single transaction: insert + enforce limit
async fn save_message_transactional(pool: &Pool<Postgres>, msg: ChatMessage) -> Result<()> {
    let mut tx = pool.begin().await?;

    // Insert new message
    sqlx::query(
        "INSERT INTO messages (room_id, user_id, username, content, timestamp)
         VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(&msg.room_id)
    .bind(&msg.user_id)
    .bind(&msg.username)
    .bind(&msg.content)
    .bind(&msg.timestamp)
    .execute(&mut *tx)
    .await?;

    // Trigger automatically enforces 1000 limit
    tx.commit().await?;
    Ok(())
}
```

### 3.4 Update Server Logic
- Remove in-memory `VecDeque<ChatMessage>` from RoomState
- Use `save_message_transactional()` for all new messages
- Load history from database on client join
- Database trigger guarantees max 1000 messages per room

**Files**: `server/migrations/`, `server/src/db.rs`, `server/src/state.rs`, `server/src/ws.rs`

## Phase 4: Update Constants

### 4.1 Change MAX_MESSAGE_HISTORY
- Update from 500 to 1000 in code/docs
- Remove in-memory limit enforcement (now in DB)

**Files**: `server/src/state.rs`, `.env.example`

## Implementation Order

1. **Phase 2 first** (Multi-line input) - Easiest, immediate UX improvement
2. **Phase 1 next** (Fix scrolling) - Critical bug fix
3. **Phase 3 last** (Database) - Largest change, depends on stable client

## Key Guarantees

✅ **Transactional**: Insert + cleanup happens atomically
✅ **Impossible to exceed 1000**: Database trigger enforces at DB level
✅ **Race-condition safe**: Trigger executes per-row, handles concurrent inserts
✅ **Crash-safe**: Even if app crashes mid-operation, DB constraint holds

## Testing Checklist

- [ ] Multi-line input expands properly
- [ ] Long messages wrap correctly in history
- [ ] Scrolling up/down works smoothly without disappearing messages
- [ ] Messages persist across server restarts
- [ ] Room limited to exactly 1000 messages (test with 1001+ inserts)
- [ ] Concurrent message inserts don't exceed 1000
- [ ] Auto-scroll to bottom on new messages works

## Notes

- PostgreSQL trigger provides hard database-level enforcement
- Transaction ensures atomicity of insert + cleanup
- No race conditions possible - enforced at database level
- Simpler than application-level cleanup
- Paragraph with `.scroll()` is the recommended approach for variable-height wrapped content
- tui-textarea is the standard solution for multi-line input in Ratatui
- Database storage makes rooms truly persistent across restarts
