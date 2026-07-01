CREATE TABLE IF NOT EXISTS settings (
    key        TEXT PRIMARY KEY NOT NULL,
    value      TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS play_history (
    id         INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    track_id   TEXT NOT NULL,
    played_at  INTEGER NOT NULL,
    source     TEXT
);
