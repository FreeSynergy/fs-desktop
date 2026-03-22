//! Database integration for the Desktop shell.
//!
//! Exposes [`DbContext`], a cheap-to-clone handle around a single shared [`FsdDb`]
//! that is opened once at Desktop startup and injected into the Dioxus context tree.
//!
//! All functions receive the shared handle instead of opening a new connection pool
//! on every call — this eliminates the `free(): corrupted unsorted chunks` crash
//! caused by SQLite FFI teardown racing with partially-initialised pools that were
//! spawned fire-and-forget.

use std::sync::Arc;

use fs_db_desktop::{FsdDb, desktop::DbWidgetSlot};

use crate::widgets::{WidgetKind, WidgetSlot};

// ── DbContext ─────────────────────────────────────────────────────────────────

/// Dioxus context holding the single shared database connection.
///
/// Opened once at Desktop startup via [`FsdDb::open`].
/// [`Clone`] is cheap — it only increments an [`Arc`] reference count.
#[derive(Clone)]
pub struct DbContext(pub Arc<FsdDb>);

// ── Sync load wrappers ────────────────────────────────────────────────────────

/// Loads the active theme from `fs-desktop.db`.
/// Falls back to `"midnight-blue"` on any error.
pub fn load_theme_from_db(db: &FsdDb) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            db.desktop().active_theme().await.unwrap_or_else(|_| "midnight-blue".to_string())
        })
    })
}

/// Loads widget slots from `fs-desktop.db`.
/// Returns an empty vec on error (caller falls back to the default layout).
pub fn load_widgets_from_db(db: &FsdDb) -> Vec<WidgetSlot> {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            db.desktop()
                .widget_slots()
                .await
                .unwrap_or_default()
                .into_iter()
                .filter_map(db_slot_to_widget)
                .collect()
        })
    })
}

/// Loads the i18n language selection from `fs-shared.db`.
/// Falls back to `"de"` on error.
pub fn load_language_from_db(db: &FsdDb) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            db.shared()
                .get_setting_or("language", "de")
                .await
                .unwrap_or_else(|_| "de".to_string())
        })
    })
}

/// Loads the wallpaper CSS from `fs-shared.db`.
/// Returns an empty string if not set (caller uses the default).
pub fn load_wallpaper_css_from_db(db: &FsdDb) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            db.shared()
                .get_setting_or("wallpaper_css", "")
                .await
                .unwrap_or_default()
        })
    })
}

// ── Async save wrappers (fire-and-forget via spawn) ───────────────────────────

/// Saves the active theme to `fs-desktop.db`.
pub fn save_theme_to_db(db: Arc<FsdDb>, name: String) {
    tokio::spawn(async move {
        let _ = db.desktop().set_active_theme(&name).await;
    });
}

/// Saves widget slots to `fs-desktop.db`.
pub fn save_widgets_to_db(db: Arc<FsdDb>, slots: Vec<WidgetSlot>) {
    tokio::spawn(async move {
        let db_slots: Vec<DbWidgetSlot> = slots
            .iter()
            .enumerate()
            .map(|(i, s)| DbWidgetSlot {
                id:         s.id,
                kind:       s.kind.as_str(),
                x:          s.x,
                y:          s.y,
                w:          s.w,
                h:          s.h,
                sort_order: i as u32,
            })
            .collect();
        let _ = db.desktop().save_widget_slots(&db_slots).await;
    });
}

/// Saves the language selection to `fs-shared.db`.
pub fn save_language_to_db(db: Arc<FsdDb>, lang: String) {
    tokio::spawn(async move {
        let _ = db.shared().set_setting("language", &lang).await;
    });
}

/// Saves the wallpaper CSS to `fs-shared.db`.
pub fn save_wallpaper_css_to_db(db: Arc<FsdDb>, css: String) {
    tokio::spawn(async move {
        let _ = db.shared().set_setting("wallpaper_css", &css).await;
    });
}

// ── Conversions ───────────────────────────────────────────────────────────────

fn db_slot_to_widget(db: DbWidgetSlot) -> Option<WidgetSlot> {
    let kind = WidgetKind::from_str(&db.kind)?;
    Some(WidgetSlot { id: db.id, kind, x: db.x, y: db.y, w: db.w, h: db.h })
}
