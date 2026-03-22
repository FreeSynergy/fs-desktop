// `fs-desktop.db` — Desktop-specific storage via SeaORM.
//
// Tables: active_theme, widget_slots, shortcuts, profile_data

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, Order, TransactionTrait,
};

use crate::{
    DbError, db_path,
    entities::{active_theme, shortcut, widget_slot},
    migration::DesktopMigrator,
};
use fs_db::{DbBackend, DbConnection};

/// Database handle for `fs-desktop.db`.
pub struct DesktopDb {
    conn: DbConnection,
}

impl DesktopDb {
    /// Open (or create) `~/.local/share/fsn/fs-desktop.db`, running migrations.
    pub async fn open() -> Result<Self, DbError> {
        let path = db_path("fs-desktop.db");
        std::fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new(".")))
            .map_err(DbError::Io)?;
        let conn = DbConnection::connect(DbBackend::Sqlite {
            path: path.to_string_lossy().into_owned(),
        })
        .await
        .map_err(|e| DbError::SeaOrm(e.to_string()))?;
        DesktopMigrator::run(conn.inner()).await?;
        Ok(Self { conn })
    }

    fn db(&self) -> &DatabaseConnection {
        self.conn.inner()
    }

    // ── Theme ─────────────────────────────────────────────────────────────────

    /// Returns the active theme name (never empty — default is `midnight-blue`).
    pub async fn active_theme(&self) -> Result<String, DbError> {
        let row = active_theme::Entity::find_by_id(1)
            .one(self.db())
            .await
            .map_err(|e| DbError::SeaOrm(e.to_string()))?;
        Ok(row.map(|m| m.name).unwrap_or_else(|| "midnight-blue".into()))
    }

    /// Persists the active theme name.
    pub async fn set_active_theme(&self, name: &str) -> Result<(), DbError> {
        let active = active_theme::ActiveModel {
            id:   Set(1),
            name: Set(name.to_string()),
        };
        active
            .update(self.db())
            .await
            .map(|_| ())
            .map_err(|e| DbError::SeaOrm(e.to_string()))
    }

    // ── Widget slots ──────────────────────────────────────────────────────────

    /// Loads all widget slots ordered by `sort_order`.
    pub async fn widget_slots(&self) -> Result<Vec<DbWidgetSlot>, DbError> {
        let rows = widget_slot::Entity::find()
            .order_by(widget_slot::Column::SortOrder, Order::Asc)
            .all(self.db())
            .await
            .map_err(|e| DbError::SeaOrm(e.to_string()))?;
        Ok(rows.into_iter().map(DbWidgetSlot::from).collect())
    }

    /// Replaces ALL widget slots with the given list (full replace on save).
    pub async fn save_widget_slots(&self, slots: &[DbWidgetSlot]) -> Result<(), DbError> {
        let tx = self.db().begin().await.map_err(|e| DbError::SeaOrm(e.to_string()))?;
        widget_slot::Entity::delete_many()
            .exec(&tx)
            .await
            .map_err(|e| DbError::SeaOrm(e.to_string()))?;
        for (i, s) in slots.iter().enumerate() {
            let active = widget_slot::ActiveModel {
                kind:       Set(s.kind.clone()),
                pos_x:      Set(s.x),
                pos_y:      Set(s.y),
                width:      Set(s.w),
                height:     Set(s.h),
                sort_order: Set(i as i64),
                ..Default::default()
            };
            active.insert(&tx).await.map_err(|e| DbError::SeaOrm(e.to_string()))?;
        }
        tx.commit().await.map_err(|e| DbError::SeaOrm(e.to_string()))
    }

    // ── Shortcuts ─────────────────────────────────────────────────────────────

    /// Returns all custom shortcut overrides.
    pub async fn shortcuts(&self) -> Result<Vec<DbShortcut>, DbError> {
        let rows = shortcut::Entity::find()
            .all(self.db())
            .await
            .map_err(|e| DbError::SeaOrm(e.to_string()))?;
        Ok(rows.into_iter().map(|r| DbShortcut {
            action_id: r.action_id,
            key_combo: r.key_combo,
        }).collect())
    }

    /// Upserts a single shortcut override.
    pub async fn set_shortcut(&self, action_id: &str, key_combo: &str) -> Result<(), DbError> {
        use sea_orm::sea_query::OnConflict;
        let active = shortcut::ActiveModel {
            action_id: Set(action_id.to_string()),
            key_combo: Set(key_combo.to_string()),
            ..Default::default()
        };
        shortcut::Entity::insert(active)
            .on_conflict(
                OnConflict::column(shortcut::Column::ActionId)
                    .update_column(shortcut::Column::KeyCombo)
                    .to_owned(),
            )
            .exec(self.db())
            .await
            .map(|_| ())
            .map_err(|e| DbError::SeaOrm(e.to_string()))
    }

    /// Removes a shortcut override (reverts to default).
    pub async fn delete_shortcut(&self, action_id: &str) -> Result<(), DbError> {
        shortcut::Entity::delete_many()
            .filter(shortcut::Column::ActionId.eq(action_id))
            .exec(self.db())
            .await
            .map(|_| ())
            .map_err(|e| DbError::SeaOrm(e.to_string()))
    }

    /// Explicitly close the connection pool.
    pub async fn close(self) -> Result<(), DbError> {
        self.conn.close().await.map_err(|e| DbError::SeaOrm(e.to_string()))
    }
}

// ── Data types ────────────────────────────────────────────────────────────────

/// A widget slot row as stored in `fs-desktop.db`.
#[derive(Debug, Clone)]
pub struct DbWidgetSlot {
    pub id:         u32,
    pub kind:       String,
    pub x:          f64,
    pub y:          f64,
    pub w:          f64,
    pub h:          f64,
    pub sort_order: u32,
}

impl From<widget_slot::Model> for DbWidgetSlot {
    fn from(m: widget_slot::Model) -> Self {
        Self {
            id:         m.id as u32,
            kind:       m.kind,
            x:          m.pos_x,
            y:          m.pos_y,
            w:          m.width,
            h:          m.height,
            sort_order: m.sort_order as u32,
        }
    }
}

/// A keyboard shortcut override row.
#[derive(Debug, Clone)]
pub struct DbShortcut {
    pub action_id: String,
    pub key_combo: String,
}
