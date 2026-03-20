//! `fsn-container-app.db` — Container App storage schema.
//!
//! Used by the Container App to persist:
//! - Service configurations (name, image, variables)
//! - Generated Quadlet files
//! - Extracted variables with type/role detection
//! - Instance names assigned to services

/// SQL to create all Container App tables. Run at Container App startup.
pub const SCHEMA: &str = r#"
-- Service configurations imported from YAML or defined manually.
CREATE TABLE IF NOT EXISTS services (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    name         TEXT    NOT NULL,
    instance     TEXT    NOT NULL DEFAULT '',
    image        TEXT    NOT NULL,
    project_id   TEXT,
    host_id      TEXT,
    yaml_source  TEXT,
    status       TEXT    NOT NULL DEFAULT 'draft',
    created_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_services_name ON services (name);

-- Sub-services (databases, caches, etc.) attached to a parent service.
CREATE TABLE IF NOT EXISTS subservices (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id INTEGER NOT NULL REFERENCES services(id) ON DELETE CASCADE,
    name       TEXT    NOT NULL,
    image      TEXT    NOT NULL,
    role       TEXT    NOT NULL  -- e.g. "database/postgres", "cache/dragonfly"
);

-- Environment variables extracted from service YAML.
CREATE TABLE IF NOT EXISTS variables (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id   INTEGER NOT NULL REFERENCES services(id) ON DELETE CASCADE,
    key          TEXT    NOT NULL,
    value        TEXT    NOT NULL DEFAULT '',
    detected_type TEXT,   -- PASSWORD, HOST, URL, PORT, etc.
    detected_role TEXT,   -- e.g. "database", "cache"
    confidence   REAL    NOT NULL DEFAULT 0.0,
    is_secret    INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_variables_service ON variables (service_id);

-- Generated Quadlet .container files.
CREATE TABLE IF NOT EXISTS quadlets (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id INTEGER NOT NULL REFERENCES services(id) ON DELETE CASCADE,
    filename   TEXT    NOT NULL,
    content    TEXT    NOT NULL,
    generated_at TEXT  NOT NULL DEFAULT (datetime('now'))
);

-- Validation results from dry-run.
CREATE TABLE IF NOT EXISTS validations (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id INTEGER NOT NULL REFERENCES services(id) ON DELETE CASCADE,
    level      TEXT    NOT NULL, -- error / warning / info
    message    TEXT    NOT NULL,
    checked_at TEXT    NOT NULL DEFAULT (datetime('now'))
);
"#;
