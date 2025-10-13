-- Add migration script here
CREATE TABLE players (
    player_id TEXT PRIMARY KEY,
    state jsonb NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TYPE player AS (
    player_id TEXT,
    state jsonb,
    created_at TIMESTAMPTZ
);