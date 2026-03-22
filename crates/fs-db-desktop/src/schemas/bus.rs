//! `fs-bus.db` — Message Bus storage schema.
//!
//! Used by the Bus subsystem to persist:
//! - Event log (audit trail of all bus events)
//! - Routing rules (TOML-based, stored as JSON)
//! - Standing orders (fire when matching service is installed)
//! - Subscriptions (service role + topic filter)

/// SQL to create all Bus tables. Run at Bus startup.
pub const SCHEMA: &str = r#"
-- Persistent event log (Audit-Log for bus events).
CREATE TABLE IF NOT EXISTS event_log (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id     TEXT    NOT NULL UNIQUE,   -- UUID
    topic        TEXT    NOT NULL,
    source_role  TEXT    NOT NULL,
    source_inst  TEXT,
    payload_json TEXT    NOT NULL DEFAULT '{}',
    delivery     TEXT    NOT NULL DEFAULT 'fire-and-forget', -- fire-and-forget / guaranteed / standing-order
    storage      TEXT    NOT NULL DEFAULT 'no-store',        -- no-store / until-ack / persistent
    acked_at     TEXT,
    created_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_event_log_topic ON event_log (topic);
CREATE INDEX IF NOT EXISTS idx_event_log_source ON event_log (source_role);

-- Subscriptions: which roles listen to which topics.
CREATE TABLE IF NOT EXISTS subscriptions (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    subscriber_role TEXT NOT NULL,
    topic_filter TEXT    NOT NULL,  -- glob pattern (e.g. "auth.*", "*")
    inst_tag     TEXT,              -- optional instance tag filter
    granted_read INTEGER NOT NULL DEFAULT 1,
    created_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_subs_role ON subscriptions (subscriber_role);

-- Routing rules: WHEN topic X AND source Y → delivery type + storage type.
CREATE TABLE IF NOT EXISTS routing_rules (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT    NOT NULL,
    topic_pattern TEXT    NOT NULL,
    source_role   TEXT,
    delivery      TEXT    NOT NULL DEFAULT 'fire-and-forget',
    storage       TEXT    NOT NULL DEFAULT 'no-store',
    priority      INTEGER NOT NULL DEFAULT 0,
    enabled       INTEGER NOT NULL DEFAULT 1
);

-- Standing orders: persistent subscriptions that fire when a matching service appears.
CREATE TABLE IF NOT EXISTS standing_orders (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT    NOT NULL,
    trigger_role  TEXT    NOT NULL,  -- fires when a service of this role is installed
    topic         TEXT    NOT NULL,
    payload_json  TEXT    NOT NULL DEFAULT '{}',
    enabled       INTEGER NOT NULL DEFAULT 1,
    created_at    TEXT    NOT NULL DEFAULT (datetime('now'))
);
"#;
