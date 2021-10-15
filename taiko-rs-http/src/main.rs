use std::sync::{Arc, Mutex};

use rocket::{Data, data::ToByteUnit};
use taiko_rs_common::{serialization::{SerializationReader, SerializationWriter}, types::Score};
use tokio::sync::OnceCell;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set, Statement, Unset, Value, FromQueryResult};

mod scores_table;

#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "How did you get here?"
}

pub use scores_table::Entity as Scores;
pub use scores_table::Model as ScoresModel;
pub use scores_table::ActiveModel as ScoresActiveModel;

pub static DATABASE:OnceCell<DatabaseConnection> = OnceCell::const_new();

#[post("/score_submit", data = "<data>")]
async fn score_submit(data:Data<'_>) -> std::io::Result<()> {
    let mut bytes:Vec<u8> = Vec::new();

    match data.open(1.gigabytes()).into_bytes().await {
        Ok(capped_bytes) => {
            capped_bytes.iter().for_each(|b|bytes.push(*b));
        }
        Err(e) => {
            println!("error reading score: {}", e);
            return Err(e);
        }
    }

    let mut reader = SerializationReader::new(bytes);
    let score: Score = reader.read();
    println!("got score: {:?}", score);

    let playmode: u8 = score.playmode.into();

    let new_score: ScoresActiveModel = ScoresActiveModel{
        username: Set(score.username),
        beatmaphash: Set(score.beatmap_hash),
        playmode: Set(playmode as i16),
        score: Set(score.score as i64),
        combo: Set(score.combo as i16),
        maxcombo: Set(score.max_combo as i16),
        hit50: Set(score.x50 as i16),
        hit100: Set(score.x100 as i16),
        hit300: Set(score.x300 as i16),
        hitgeki: Set(score.xgeki as i16),
        hitkatu: Set(score.xkatu as i16),
        hitmiss: Set(score.xmiss as i16),
        accuracy: Set(score.accuracy),
        ..Default::default()
    };

    let res = new_score.insert(DATABASE.get().unwrap()).await.unwrap();

    println!("InsertResult: {:?}", res.id);

    Ok(())
}

#[post("/get_scores", data = "<data>")]
async fn get_scores(data:Data<'_>) -> std::io::Result<Vec<u8>> {
    let mut bytes:Vec<u8> = Vec::new();

    match data.open(1.gigabytes()).into_bytes().await {
        Ok(capped_bytes) => {
            capped_bytes.iter().for_each(|b|bytes.push(*b));
        }
        Err(e) => {
            println!("error reading score: {}", e);
            return Err(e);
        }
    }

    let mut reader = SerializationReader::new(bytes);
    let hash: String = reader.read();
    println!("got hash: {}", hash);

    let mut writer = SerializationWriter::new();
    
    let scores: Vec<ScoresModel> = Scores::find().filter(scores_table::Column::Beatmaphash.eq(hash)).all(DATABASE.get().unwrap()).await.unwrap();

    let new_scores: Vec<Score> = scores.iter().map(|score| {
        let mut new_score: Score = Score::new(score.beatmaphash.clone(), score.username.clone(), (score.playmode as u8).into());

        new_score.score = score.score as u64;
        new_score.combo = score.combo as u16;
        new_score.max_combo = score.maxcombo as u16;
        new_score.x300 = score.hit300 as u16;
        new_score.x100 = score.hit100 as u16;
        new_score.x50 = score.hit50 as u16;
        new_score.xgeki = score.hitgeki as u16;
        new_score.xkatu = score.hitkatu as u16;
        new_score.xmiss = score.hitmiss as u16;
        new_score.accuracy = score.accuracy;

        return new_score;
    }).collect();

    writer.write(new_scores);

    Ok(writer.data())
}

#[launch]
pub async fn rocket() -> _ {
    //Connect to sex fuck
    let db = sea_orm::Database::connect("postgres://taiko-rs:uwu@192.168.0.201:5434/taiko-rs")
        .await
        .expect("Error connecting to database");
    
    DATABASE.set(db).unwrap();

    rocket::build().mount("/", routes![index, score_submit, get_scores])
}
