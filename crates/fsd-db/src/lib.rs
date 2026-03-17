//! FreeSynergy.Desktop — SQLite storage layer (SeaORM-based).
//!
//! Manages four databases:
//! - `fsn-desktop.db`: widget positions, active theme, shortcuts
//! - `fsn-shared.db`: cross-program settings, i18n selection, audit log
//! - `fsn-conductor.db`: service configurations, quadlets, variables
//! - `fsn-store.db`: installed packages, catalog cache, download queue
//!
//! Also holds SQL schema definitions for the Node-side databases (core, bus).
//!
//! # Usage
//! ```rust,ignore
//! let db = FsdDb::open().await?;
//! db.desktop().set_active_theme("midnight-blue").await?;
//! db.shared().set_setting("language", "de").await?;
//! ```

pub mod conductor_db;
pub mod desktop;
pub mod entities;
pub mod migration;
pub mod package_registry;
pub mod schemas;
pub mod shared;
pub mod store_db;

pub use conductor_db::ConductorDb;
pub use desktop::DesktopDb;
pub use shared::SharedDb;
pub use store_db::StoreDb;

use std::path::PathBuf;

/// Combined handle for all Desktop-side databases.
pub struct FsdDb {
    desktop:   DesktopDb,
    shared:    SharedDb,
    conductor: ConductorDb,
    store:     StoreDb,
}

impl FsdDb {
    /// Open (or create) all four databases at their default paths.
    pub async fn open() -> Result<Self, DbError> {
        let desktop   = DesktopDb::open().await?;
        let shared    = SharedDb::open().await?;
        let conductor = ConductorDb::open().await?;
        let store     = StoreDb::open().await?;
        Ok(Self { desktop, shared, conductor, store })
    }

    pub fn desktop(&self)   -> &DesktopDb   { &self.desktop   }
    pub fn shared(&self)    -> &SharedDb    { &self.shared    }
    pub fn conductor(&self) -> &ConductorDb { &self.conductor }
    pub fn store(&self)     -> &StoreDb     { &self.store     }
}

/// Returns `~/.local/share/fsn/<name>` as the path for an FSN database.
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
