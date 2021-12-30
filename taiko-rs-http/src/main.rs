use std::collections::HashMap;
use std::io::{Error, ErrorKind};

use argon2::{
    password_hash::{
        PasswordHash, 
        PasswordVerifier
    },
    Argon2
};

use rocket::{Data, data::ToByteUnit};
use taiko_rs_common::{serialization::{SerializationReader, SerializationWriter}, types::Score};
use tokio::sync::OnceCell;
use sea_orm::{DbBackend, ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, Statement, FromQueryResult};

#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "How did you get here?"
}

use taiko_rs_common::prelude::*;



pub static DATABASE:OnceCell<DatabaseConnection> = OnceCell::const_new();

#[post("/score_submit", data = "<data>")]
async fn score_submit(data:Data<'_>) -> std::io::Result<()> {
    println!("submit shit idk");
    let mut bytes:Vec<u8> = Vec::new();

    match data.open(1u32.gigabytes()).into_bytes().await {
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
    let password = reader.read_string();

    let playmode: u8 = score.playmode.into();

    let new_score: ScoresActiveModel = ScoresActiveModel{
        username: Set(score.username.clone()),
        beatmap_hash: Set(score.beatmap_hash),
        playmode: Set(playmode as i16),
        score: Set(score.score as i64),
        combo: Set(score.combo as i16),
        max_combo: Set(score.max_combo as i16),
        hit_50: Set(score.x50 as i16),
        hit_100: Set(score.x100 as i16),
        hit_300: Set(score.x300 as i16),
        hit_geki: Set(score.xgeki as i16),
        hit_katu: Set(score.xkatu as i16),
        hit_miss: Set(score.xmiss as i16),
        accuracy: Set(score.accuracy),
        ..Default::default()
    };

    let user: Option<users_table::Model> = users_table::Entity::find()
        .filter(users_table::Column::Username.eq(score.username.clone()))
        .one(DATABASE.get().unwrap())
        .await
        .unwrap();

    let user_id;

    match user {
        None => {
            // User not found
            return Err(Error::new(ErrorKind::NotFound, "User not found!"));
        }
        Some(user) => {
            user_id = user.user_id;

            let argon2 = Argon2::default();

            let parsed_hash = PasswordHash::new(&user.password).unwrap();
            if !argon2.verify_password(password.as_ref(), &parsed_hash).is_ok() {
                return Err(Error::new(ErrorKind::PermissionDenied, "Your password is incorrect"));
            }
        }
    };

    let _ = new_score.insert(DATABASE.get().unwrap()).await.unwrap();

    recalc_user(score.username, user_id, score.playmode).await;

    Ok(())
}

async fn recalc_user(username: String, user_id: i32, mode: PlayMode) {
    println!("Recalcing user!");
    let scores: Vec<ScoresModel> = Scores::find().filter(scores_table::Column::Playmode.eq(mode as i16)).filter(scores_table::Column::Username.eq(username)).all(DATABASE.get().unwrap()).await.unwrap();

    let mut ranked_score = 0;
    let mut total_score = 0;
    let mut total_accuracy = 0.0;
    let play_count = scores.len() as i32;

    let mut best_scores: HashMap<String, ScoresModel> = HashMap::new();

    for score in scores {
        total_score += score.score;

        if let Some(best) = best_scores.get(score.beatmap_hash.clone().as_str()) {
            if best.score < score.score {
                best_scores.insert(score.beatmap_hash.clone(), score.clone());
            }
        } else {
            best_scores.insert(score.beatmap_hash.clone(), score.clone());
        }
    }

    for (_hash, score) in best_scores.clone() {
        ranked_score += score.score;
        total_accuracy += score.accuracy;
    }

    let accuracy = total_accuracy / best_scores.len() as f64;

    let user_data: Option<user_data_table::Model>;

    match user_data_table::Entity::find()
        .filter(user_data_table::Column::Mode.eq(mode as i16))
        .filter(user_data_table::Column::UserId.eq(user_id))
        .one(DATABASE.get().unwrap()).await {
        Ok(user_data_a) => {
            println!("query ok");
            user_data = user_data_a;
        },
        Err(e) => {
            println!("error: {}", e);
            panic!();
        }
    }

    match user_data {
        Some(user_data) => {
            let mut user_data: user_data_table::ActiveModel = user_data.into();
            user_data.total_score = Set(total_score);
            user_data.ranked_score = Set(ranked_score);
            user_data.accuracy = Set(accuracy);
            user_data.play_count = Set(play_count);
            user_data.mode = Set(mode as i16);

            match user_data.update(DATABASE.get().unwrap()).await {
                Ok(_) => println!("update ok"),
                Err(e) => println!("error: {}", e)
            }
        }
        None => {
            let user_data = user_data_table::ActiveModel {
                user_id: Set(user_id),
                ranked_score: Set(ranked_score),
                total_score: Set(total_score),
                accuracy: Set(accuracy),
                play_count: Set(play_count),
                mode: Set(mode as i16),
                ..Default::default()
            };

            match user_data.insert(DATABASE.get().unwrap()).await {
                Ok(_) => println!("insert ok"),
                Err(e) => println!("error: {}", e)
            }
        }
    };
}

#[post("/get_scores", data = "<data>")]
async fn get_scores(data:Data<'_>) -> std::io::Result<Vec<u8>> {
    let mut bytes:Vec<u8> = Vec::new();

    match data.open(1u32.gigabytes()).into_bytes().await {
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
    let mode: PlayMode = reader.read();
    println!("got hash: {}", hash);

    let mut writer = SerializationWriter::new();
    
    let scores: Vec<ScoresModel> = Scores::find()
        .filter(scores_table::Column::Playmode.eq(mode as i16))
        .filter(scores_table::Column::BeatmapHash.eq(hash))
        .all(DATABASE.get().unwrap())
        .await
        .unwrap();

    let new_scores: Vec<Score> = scores.iter().map(|score| {
        let mut new_score:Score = Score::new(score.beatmap_hash.clone(), score.username.clone(), (score.playmode as u8).into());

        new_score.score = score.score as u64;
        new_score.combo = score.combo as u16;
        new_score.max_combo = score.max_combo as u16;
        new_score.x300 = score.hit_300 as u16;
        new_score.x100 = score.hit_100 as u16;
        new_score.x50 = score.hit_50 as u16;
        new_score.xgeki = score.hit_geki as u16;
        new_score.xkatu = score.hit_katu as u16;
        new_score.xmiss = score.hit_miss as u16;
        new_score.accuracy = score.accuracy;

        return new_score;
    }).collect();

    let mut filtered_scores: HashMap<String, Score> = HashMap::new();

    for score in new_scores {
        if let Some(best) = filtered_scores.get(&*score.username.clone()) {
            if best.score < score.score {
                filtered_scores.insert(score.username.clone(), score.clone());
            }
        } else {
            filtered_scores.insert(score.username.clone(), score.clone());
        }
    }

    let mut filtered_scores_vec: Vec<Score> = Vec::new();

    for (_username, score) in filtered_scores {
        filtered_scores_vec.push(score);
    }

    writer.write(filtered_scores_vec);

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
