use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "scores")]
pub struct Model {
    pub username: String,
    #[sea_orm(column_name = "user_id")]
    pub user_id: i32,
    pub beatmap_hash: String,
    #[sea_orm(primary_key)]
    pub score_id: i64,
    pub playmode: i16,
    pub score: i64,
    pub combo: i16,
    #[sea_orm(column_name = "max_combo")]
    pub max_combo: i16,
    #[sea_orm(column_name = "hit_50")]
    pub hit_50: i16,
    #[sea_orm(column_name = "hit_100")]
    pub hit_100: i16,
    #[sea_orm(column_name = "hit_300")]
    pub hit_300: i16,
    #[sea_orm(column_name = "hit_geki")]
    pub hit_geki: i16,
    #[sea_orm(column_name = "hit_katu")]
    pub hit_katu: i16,
    #[sea_orm(column_name = "hit_miss")]
    pub hit_miss: i16,
    pub accuracy: f64,
    pub game: String
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation { }

impl ActiveModelBehavior for ActiveModel { }