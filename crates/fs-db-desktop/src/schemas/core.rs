//! `fs-core.db` — Node core storage schema.
//!
//! Used by the Node program to persist:
//! - Hosts and their status
//! - Projects and their configuration
//! - Invitations (tokens, encrypted TOML)
//! - Federation membership

/// SQL to create all Node core tables. Run at Node startup.
pub const SCHEMA: &str = r"
-- Managed hosts.
CREATE TABLE IF NOT EXISTS hosts (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL UNIQUE,
    domain      TEXT    NOT NULL,
    ip_address  TEXT,
    ssh_port    INTEGER NOT NULL DEFAULT 22,
    status      TEXT    NOT NULL DEFAULT 'unknown',  -- online / offline / unknown
    project_id  INTEGER,
    notes       TEXT,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- Projects (logical groupings of services).
CREATE TABLE IF NOT EXISTS projects (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL UNIQUE,
    domain      TEXT,
    status      TEXT    NOT NULL DEFAULT 'draft',  -- draft / active / archived
    description TEXT,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- Invitations for new nodes to join a project.
CREATE TABLE IF NOT EXISTS invitations (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    token           TEXT    NOT NULL UNIQUE,
    project_id      INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    role            TEXT    NOT NULL DEFAULT 'member',
    encrypted_toml  TEXT,   -- age-encrypted join config
    port            INTEGER,
    expires_at      TEXT,
    used_at         TEXT,
    created_at      TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- Federation peers (other FreeSynergy networks this node has joined).
CREATE TABLE IF NOT EXISTS federation_peers (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL,
    domain      TEXT    NOT NULL UNIQUE,
    auth_broker TEXT,   -- URL of the IAM broker
    status      TEXT    NOT NULL DEFAULT 'pending',  -- pending / active / suspended
    joined_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- Rights granted between this node and federation peers.
CREATE TABLE IF NOT EXISTS federation_rights (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    peer_id   INTEGER NOT NULL REFERENCES federation_peers(id) ON DELETE CASCADE,
    direction TEXT    NOT NULL,  -- inbound / outbound
    right     TEXT    NOT NULL,  -- read / write / execute / search
    scope     TEXT    NOT NULL DEFAULT '*'
);
";
