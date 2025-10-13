-- Add nodes table
CREATE TABLE nodes (
    peer_id TEXT PRIMARY KEY,
    ip TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TYPE node AS (
    peer_id TEXT,
    ip TEXT,
    created_at TIMESTAMPTZ
);