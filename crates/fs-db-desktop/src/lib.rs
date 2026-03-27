#![deny(clippy::all, clippy::pedantic, warnings)]
//! FreeSynergy.Desktop — `SQLite` storage layer (`SeaORM`-based).
//!
//! Manages four databases:
//! - `fs-desktop.db`: widget positions, active theme, shortcuts
//! - `fs-shared.db`: cross-program settings, i18n selection, audit log
//! - `fs-container-app.db`: service configurations, quadlets, variables
//! - `fs-store.db`: installed packages, catalog cache, download queue
//!
//! Also holds SQL schema definitions for the Node-side databases (core, bus).
//!
//! # Usage
//! ```rust,ignore
//! let db = FsdDb::open().await?;
//! db.desktop().set_active_theme("midnight-blue").await?;
//! db.shared().set_setting("language", "de").await?;
//! ```

pub mod container_db;
pub mod desktop;
pub mod entities;
pub mod migration;
pub mod package_registry;
pub mod schemas;
pub mod shared;
pub mod store_db;

pub use container_db::ContainerDb;
pub use desktop::DesktopDb;
pub use shared::SharedDb;
pub use store_db::StoreDb;

use std::path::PathBuf;

/// Combined handle for all Desktop-side databases.
pub struct FsdDb {
    desktop: DesktopDb,
    shared: SharedDb,
    container: ContainerDb,
    store: StoreDb,
}

impl FsdDb {
    /// Open (or create) all four databases at their default paths.
    ///
    /// # Errors
    /// Returns [`DbError`] if any database file cannot be opened or migrations fail.
    pub async fn open() -> Result<Self, DbError> {
        let desktop = DesktopDb::open().await?;
        let shared = SharedDb::open().await?;
        let container = ContainerDb::open().await?;
        let store = StoreDb::open().await?;
        Ok(Self {
            desktop,
            shared,
            container,
            store,
        })
    }

    #[must_use]
    pub fn desktop(&self) -> &DesktopDb {
        &self.desktop
    }
    #[must_use]
    pub fn shared(&self) -> &SharedDb {
        &self.shared
    }
    #[must_use]
    pub fn container(&self) -> &ContainerDb {
        &self.container
    }
    #[must_use]
    pub fn store(&self) -> &StoreDb {
        &self.store
    }

    /// Explicitly close all four connection pools.
    ///
    /// Call this on clean shutdown to avoid heap corruption from `SQLite` FFI
    /// teardown racing with the Tokio runtime shutdown.
    pub async fn close(self) {
        let _ = self.desktop.close().await;
        let _ = self.shared.close().await;
        let _ = self.container.close().await;
        let _ = self.store.close().await;
    }
}

/// Returns `~/.local/share/fsn/<name>` as the path for an FSN database.
#[must_use]
pub fn db_path(filename: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".local/share/fsn").join(filename)
}

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("SeaORM error: {0}")]
    SeaOrm(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
