// SeaORM entity: widget_slots.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "widget_slots")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id:         i64,
    pub kind:       String,
    pub pos_x:      f64,
    pub pos_y:      f64,
    pub width:      f64,
    pub height:     f64,
    pub sort_order: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
