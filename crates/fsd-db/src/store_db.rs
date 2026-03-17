// `fsn-store.db` — Store-specific storage.
//
// Tables: installed_packages, catalog_cache, download_queue
// Schema is idempotent (CREATE TABLE IF NOT EXISTS).

use sea_orm::DatabaseConnection;

use crate::{DbError, db_path, schemas};
use fsn_db::{DbBackend, DbConnection};

/// Database handle for `fsn-store.db`.
pub struct StoreDb {
    conn: DbConnection,
}

impl StoreDb {
    /// Open (or create) `~/.local/share/fsn/fsn-store.db`, applying the schema.
    pub async fn open() -> Result<Self, DbError> {
        let path = db_path("fsn-store.db");
        std::fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new(".")))
            .map_err(DbError::Io)?;
        let conn = DbConnection::connect(DbBackend::Sqlite {
            path: path.to_string_lossy().into_owned(),
        })
        .await
        .map_err(|e| DbError::SeaOrm(e.to_string()))?;
        conn.apply_schema(schemas::store::SCHEMA)
            .await
            .map_err(|e| DbError::SeaOrm(e.to_string()))?;
        Ok(Self { conn })
    }

    /// Access the underlying SeaORM connection for raw queries.
    pub fn db(&self) -> &DatabaseConnection {
        self.conn.inner()
    }
}
