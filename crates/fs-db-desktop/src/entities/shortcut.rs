// SeaORM entity: shortcuts.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "shortcuts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id:        i64,
    pub action_id: String,
    pub key_combo: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
