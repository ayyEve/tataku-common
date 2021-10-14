use std::sync::{Arc, Mutex};

use rocket::{Data, data::ToByteUnit};
use taiko_rs_common::{serialization::{SerializationReader, SerializationWriter}, types::Score};

#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "How did you get here?"
}

lazy_static::lazy_static! {
    pub static ref SCORES:Arc<Mutex<Vec<Score>>> = Arc::new(Mutex::new(Vec::new()));
}

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

    let mut lock = SCORES.lock().unwrap();
    lock.push(score);

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

    let lock = SCORES.lock().unwrap();
    
    let mut temp: Vec<Score> = Vec::new();
    for i in lock.iter() {
        let score: Score = i.clone();

        if score.beatmap_hash != hash { continue }

        temp.push(score);
    }
    writer.write(temp);

    Ok(writer.data())
}

#[launch]
pub fn rocket() -> _ {
    rocket::build().mount("/", routes![index, score_submit, get_scores])
}
