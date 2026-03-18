//! `fsn-lenses.db` — Lens storage schema (saved lenses + cached items).

/// SQL to create all Lenses tables. Run at Lenses startup.
pub const SCHEMA: &str = r#"
-- Saved lenses (each lens is a named search query).
CREATE TABLE IF NOT EXISTS lenses (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    name           TEXT    NOT NULL,
    query          TEXT    NOT NULL,
    last_refreshed TEXT,
    created_at     TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- Cached result items for each lens (refreshed via bus queries).
CREATE TABLE IF NOT EXISTS lens_items (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    lens_id    INTEGER NOT NULL REFERENCES lenses(id) ON DELETE CASCADE,
    role       TEXT    NOT NULL,  -- wiki / chat / git / map / tasks / iam / other:xxx
    summary    TEXT    NOT NULL,
    link       TEXT,
    source     TEXT    NOT NULL,
    fetched_at TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_lens_items_lens ON lens_items (lens_id);
"#;
