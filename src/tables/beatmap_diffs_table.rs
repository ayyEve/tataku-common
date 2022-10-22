use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "beatmap_diffs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub map_hash: String,
    #[sea_orm(primary_key)]
    pub mode: String,
    #[sea_orm(primary_key)]
    pub mods: String,
    #[sea_orm(primary_key)]
    pub speed: i32,
    
    pub diff: f32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation { }

impl ActiveModelBehavior for ActiveModel { }