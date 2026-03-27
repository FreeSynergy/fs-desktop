// `fs-shared.db` — cross-program shared storage via SeaORM.
//
// Tables: settings (KV), audit_log

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, Order, QueryOrder,
    QuerySelect,
};

use crate::{
    db_path,
    entities::{audit, setting},
    migration::SharedMigrator,
    DbError,
};
use fs_db::{DbBackend, DbConnection};

/// Database handle for `fs-shared.db`.
pub struct SharedDb {
    conn: DbConnection,
}

impl SharedDb {
    /// Open (or create) `~/.local/share/fsn/fs-shared.db`, running migrations.
    ///
    /// # Errors
    /// Returns [`DbError`] if the database file cannot be opened or migrations fail.
    pub async fn open() -> Result<Self, DbError> {
        let path = db_path("fs-shared.db");
        std::fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new(".")))
            .map_err(DbError::Io)?;
        let conn = DbConnection::connect(DbBackend::Sqlite {
            path: path.to_string_lossy().into_owned(),
        })
        .await
        .map_err(|e| DbError::SeaOrm(e.to_string()))?;
        SharedMigrator::run(conn.inner()).await?;
        Ok(Self { conn })
    }

    fn db(&self) -> &DatabaseConnection {
        self.conn.inner()
    }

    // ── Settings ──────────────────────────────────────────────────────────────

    /// Gets a setting value by key. Returns `None` if not set.
    ///
    /// # Errors
    /// Returns [`DbError`] if the database query fails.
    pub async fn get_setting(&self, key: &str) -> Result<Option<String>, DbError> {
        let row = setting::Entity::find_by_id(key.to_string())
            .one(self.db())
            .await
            .map_err(|e| DbError::SeaOrm(e.to_string()))?;
        Ok(row.map(|m| m.value))
    }

    /// Gets a setting value, returning `default` if not set.
    ///
    /// # Errors
    /// Returns [`DbError`] if the database query fails.
    pub async fn get_setting_or(&self, key: &str, default: &str) -> Result<String, DbError> {
        Ok(self
            .get_setting(key)
            .await?
            .unwrap_or_else(|| default.to_string()))
    }

    /// Upserts a setting.
    ///
    /// # Errors
    /// Returns [`DbError`] if the database upsert fails.
    pub async fn set_setting(&self, key: &str, value: &str) -> Result<(), DbError> {
        use sea_orm::sea_query::OnConflict;
        let now = unix_now();
        let active = setting::ActiveModel {
            key: Set(key.to_string()),
            value: Set(value.to_string()),
            updated_at: Set(now),
        };
        setting::Entity::insert(active)
            .on_conflict(
                OnConflict::column(setting::Column::Key)
                    .update_columns([setting::Column::Value, setting::Column::UpdatedAt])
                    .to_owned(),
            )
            .exec(self.db())
            .await
            .map(|_| ())
            .map_err(|e| DbError::SeaOrm(e.to_string()))
    }

    /// Deletes a setting (resets to default behavior).
    ///
    /// # Errors
    /// Returns [`DbError`] if the database delete fails.
    pub async fn delete_setting(&self, key: &str) -> Result<(), DbError> {
        setting::Entity::delete_by_id(key.to_string())
            .exec(self.db())
            .await
            .map(|_| ())
            .map_err(|e| DbError::SeaOrm(e.to_string()))
    }

    // ── Audit log ─────────────────────────────────────────────────────────────

    /// Appends an audit log entry.
    ///
    /// # Errors
    /// Returns [`DbError`] if the database insert fails.
    pub async fn audit(
        &self,
        actor: &str,
        action: &str,
        target: Option<&str>,
        outcome: &str,
    ) -> Result<(), DbError> {
        let active = audit::ActiveModel {
            actor: Set(actor.to_string()),
            action: Set(action.to_string()),
            target: Set(target.map(ToString::to_string)),
            outcome: Set(outcome.to_string()),
            created_at: Set(unix_now()),
            ..Default::default()
        };
        active
            .insert(self.db())
            .await
            .map(|_| ())
            .map_err(|e| DbError::SeaOrm(e.to_string()))
    }

    /// Returns the most recent N audit log entries.
    ///
    /// # Errors
    /// Returns [`DbError`] if the database query fails.
    pub async fn recent_audit(&self, limit: u32) -> Result<Vec<AuditEntry>, DbError> {
        let rows = audit::Entity::find()
            .order_by(audit::Column::CreatedAt, Order::Desc)
            .limit(u64::from(limit))
            .all(self.db())
            .await
            .map_err(|e| DbError::SeaOrm(e.to_string()))?;
        Ok(rows.into_iter().map(AuditEntry::from).collect())
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

// ── Helpers ───────────────────────────────────────────────────────────────────

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().cast_signed())
        .unwrap_or(0)
}

// ── Data types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub actor: String,
    pub action: String,
    pub target: Option<String>,
    pub outcome: String,
    pub created_at: i64,
}

impl From<audit::Model> for AuditEntry {
    fn from(m: audit::Model) -> Self {
        Self {
            actor: m.actor,
            action: m.action,
            target: m.target,
            outcome: m.outcome,
            created_at: m.created_at,
        }
    }
}
