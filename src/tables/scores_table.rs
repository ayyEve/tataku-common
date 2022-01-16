use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "scores")]
pub struct Model {
    pub username: String,
    pub beatmap_hash: String,
    #[sea_orm(primary_key)]
    pub id: i64,
    pub playmode: i16,
    pub score: i64,
    pub combo: i16,
    pub max_combo: i16,
    pub hit_50: i16,
    pub hit_100: i16,
    pub hit_300: i16,
    pub hit_geki: i16,
    pub hit_katu: i16,
    pub hit_miss: i16,
    pub accuracy: f64
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation { }

impl ActiveModelBehavior for ActiveModel { }