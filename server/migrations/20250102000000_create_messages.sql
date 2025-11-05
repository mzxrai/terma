-- Create messages table
CREATE TABLE IF NOT EXISTS messages (
    id BIGSERIAL PRIMARY KEY,
    room_id TEXT NOT NULL REFERENCES rooms(id),
    user_id TEXT NOT NULL,
    username TEXT NOT NULL,
    content TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_messages_room_timestamp
    ON messages(room_id, timestamp DESC);

-- Trigger function to enforce 1000 message limit per room
CREATE OR REPLACE FUNCTION enforce_message_limit()
RETURNS TRIGGER AS $$
BEGIN
    DELETE FROM messages
    WHERE id IN (
        SELECT id FROM messages
        WHERE room_id = NEW.room_id
        ORDER BY timestamp DESC
        LIMIT -1 OFFSET 1000
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to automatically enforce the limit
CREATE TRIGGER enforce_room_message_limit
    AFTER INSERT ON messages
    FOR EACH ROW
    EXECUTE FUNCTION enforce_message_limit();
