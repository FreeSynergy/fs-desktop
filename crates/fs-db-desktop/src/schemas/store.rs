//! `fs-store.db` — Store-specific storage schema.
//!
//! Used by the Store program to persist:
//! - Installed packages with version and status
//! - Cached catalog metadata
//! - Download queue

/// SQL to create all Store tables. Run at Store startup.
pub const SCHEMA: &str = r#"
-- Installed packages (languages, themes, widgets, modules, etc.).
CREATE TABLE IF NOT EXISTS installed_packages (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    namespace    TEXT    NOT NULL,  -- "Node", "Desktop", etc.
    name         TEXT    NOT NULL,
    version      TEXT    NOT NULL,
    kind         TEXT    NOT NULL,  -- language / theme / widget / module
    install_path TEXT,
    status       TEXT    NOT NULL DEFAULT 'installed',  -- installing / installed / error / removed
    installed_at TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_pkg_ns_name ON installed_packages (namespace, name);

-- Cached catalog entries (from remote TOML, refreshed periodically).
CREATE TABLE IF NOT EXISTS catalog_cache (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    namespace     TEXT    NOT NULL,
    name          TEXT    NOT NULL,
    version       TEXT    NOT NULL,
    kind          TEXT    NOT NULL,
    description   TEXT,
    tags          TEXT,   -- JSON array
    download_url  TEXT,
    checksum      TEXT,
    cached_at     TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_catalog_ns ON catalog_cache (namespace);

-- Download queue for async package installation.
CREATE TABLE IF NOT EXISTS download_queue (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    namespace  TEXT    NOT NULL,
    name       TEXT    NOT NULL,
    version    TEXT    NOT NULL,
    url        TEXT    NOT NULL,
    status     TEXT    NOT NULL DEFAULT 'pending',  -- pending / downloading / done / error
    error      TEXT,
    queued_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);
"#;
