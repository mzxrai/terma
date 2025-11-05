-- Create rooms table
CREATE TABLE IF NOT EXISTS rooms (
    id TEXT PRIMARY KEY NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_rooms_created_at ON rooms(created_at);
