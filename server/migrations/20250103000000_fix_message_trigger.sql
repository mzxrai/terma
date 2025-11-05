-- Fix the message limit trigger function (remove LIMIT -1 which is not supported in PostgreSQL)
CREATE OR REPLACE FUNCTION enforce_message_limit()
RETURNS TRIGGER AS $$
BEGIN
    DELETE FROM messages
    WHERE id IN (
        SELECT id FROM messages
        WHERE room_id = NEW.room_id
        ORDER BY timestamp ASC
        OFFSET 1000
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
