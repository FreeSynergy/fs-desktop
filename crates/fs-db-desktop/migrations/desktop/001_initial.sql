-- Desktop database: widget_slots, active_theme, shortcuts, profile_data

CREATE TABLE IF NOT EXISTS active_theme (
    id   INTEGER PRIMARY KEY CHECK (id = 1),
    name TEXT    NOT NULL DEFAULT 'midnight-blue'
);

INSERT OR IGNORE INTO active_theme (id, name) VALUES (1, 'midnight-blue');

CREATE TABLE IF NOT EXISTS widget_slots (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    kind       TEXT    NOT NULL,
    pos_x      REAL    NOT NULL DEFAULT 0,
    pos_y      REAL    NOT NULL DEFAULT 0,
    width      REAL    NOT NULL DEFAULT 200,
    height     REAL    NOT NULL DEFAULT 150,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS shortcuts (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    action_id TEXT    NOT NULL UNIQUE,
    key_combo TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS profile_data (
    id           INTEGER PRIMARY KEY CHECK (id = 1),
    username     TEXT NOT NULL DEFAULT '',
    display_name TEXT NOT NULL DEFAULT '',
    avatar_url   TEXT,
    bio          TEXT,
    links        TEXT
);

INSERT OR IGNORE INTO profile_data (id) VALUES (1);
