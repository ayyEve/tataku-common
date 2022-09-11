use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "screenshots")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub screenshot_id: i64,
    pub user_id: i32,
    pub views: i32
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation { }

impl ActiveModelBehavior for ActiveModel { }