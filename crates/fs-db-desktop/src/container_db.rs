// `fs-container-app.db` — Container App-specific storage.
//
// Tables: services, subservices, variables, quadlets, validations
// Schema is idempotent (CREATE TABLE IF NOT EXISTS).

use sea_orm::DatabaseConnection;

use crate::{db_path, schemas, DbError};
use fs_db::{DbBackend, DbConnection};

/// Database handle for `fs-container-app.db`.
pub struct ContainerDb {
    conn: DbConnection,
}

impl ContainerDb {
    /// Open (or create) `~/.local/share/fsn/fs-container-app.db`, applying the schema.
    ///
    /// # Errors
    /// Returns [`DbError`] if the database file cannot be opened or the schema fails.
    pub async fn open() -> Result<Self, DbError> {
        let path = db_path("fs-container-app.db");
        std::fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new(".")))
            .map_err(DbError::Io)?;
        let conn = DbConnection::connect(DbBackend::Sqlite {
            path: path.to_string_lossy().into_owned(),
        })
        .await
        .map_err(|e| DbError::SeaOrm(e.to_string()))?;
        conn.apply_schema(schemas::container::SCHEMA)
            .await
            .map_err(|e| DbError::SeaOrm(e.to_string()))?;
        Ok(Self { conn })
    }

    /// Access the underlying `SeaORM` connection for raw queries.
    #[must_use]
    pub fn db(&self) -> &DatabaseConnection {
        self.conn.inner()
    }

    /// Explicitly close the connection pool.
    ///
    /// # Errors
    /// Returns [`DbError`] if the pool cannot be closed cleanly.
    pub async fn close(self) -> Result<(), DbError> {
        self.conn
            .close()
            .await
            .map_err(|e| DbError::SeaOrm(e.to_string()))
    }
}
