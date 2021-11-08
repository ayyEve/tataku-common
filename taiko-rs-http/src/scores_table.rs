use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "scores")]
pub struct Model {
    pub username: String,
    pub beatmaphash: String,
    #[sea_orm(primary_key)]
    pub id: i64,
    pub playmode: i16,
    pub score: i64,
    pub combo: i16,
    pub maxcombo: i16,
    pub hit50: i16,
    pub hit100: i16,
    pub hit300: i16,
    pub hitgeki: i16,
    pub hitkatu: i16,
    pub hitmiss: i16,
    pub accuracy: f64
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation { }

impl ActiveModelBehavior for ActiveModel { }