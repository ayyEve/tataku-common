use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "beatmaps")]
pub struct Model {
    /// tataku beatmap id
    #[sea_orm(primary_key)]
    pub beatmap_id: i64,
    pub beatmap_hash: String,

    pub original_game_set_id: Option<i32>,
    pub original_game_map_id: Option<i32>,

    /// what game is this from (osu, quaver, etc)
    pub game: String,

    pub title: String,
    pub artist: String,
    
    pub title_unicode: Option<String>,
    pub artist_unicode: Option<String>,

    pub creator: String,
    pub difficulty_name: String, 


    // these will go into their own table at some point
    // /// what playmode is this map data for? (osu, taiko, mania, etc)
    // pub playmode: String,
    // pub difficulty_value: f32,

    /// who actually submitted this info?
    /// so we know who to blame if its incorrect :^)
    pub info_submitter_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation { }

impl ActiveModelBehavior for ActiveModel { }
