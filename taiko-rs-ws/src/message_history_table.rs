use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "message_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub userid: i64,
    pub channel: String,
    pub contents: String
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation { }

impl ActiveModelBehavior for ActiveModel { }