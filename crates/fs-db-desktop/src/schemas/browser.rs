//! `fs-browser.db` — Browser storage schema (bookmarks, history, downloads).

/// SQL to create all Browser tables. Run at Browser startup.
pub const SCHEMA: &str = r#"
-- Saved bookmarks.
CREATE TABLE IF NOT EXISTS bookmarks (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    title      TEXT    NOT NULL,
    url        TEXT    NOT NULL,
    created_at TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_bookmarks_url ON bookmarks (url);

-- Browser history (all visited URLs, duplicates allowed).
CREATE TABLE IF NOT EXISTS history (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    title      TEXT    NOT NULL,
    url        TEXT    NOT NULL,
    visited_at TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_history_url ON history (url);
CREATE INDEX IF NOT EXISTS idx_history_visited ON history (visited_at);

-- Downloads intercepted by the browser and saved to S3.
CREATE TABLE IF NOT EXISTS downloads (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    filename   TEXT    NOT NULL,
    url        TEXT    NOT NULL,
    s3_path    TEXT    NOT NULL,
    status     TEXT    NOT NULL DEFAULT 'pending',  -- pending / saving / done / error
    error_msg  TEXT,
    started_at TEXT    NOT NULL DEFAULT (datetime('now')),
    finished_at TEXT
);
"#;
