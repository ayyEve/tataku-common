use std::{collections::HashMap, sync::{Arc, Mutex}};

use taiko_rs_common::serialization::*;
use crate::gameplay::Score;

const SCORE_DATABASE_FILE:&str = "scores.db";

lazy_static::lazy_static! {
    /// SCORES_CACHE.get(.osu_hash) = list of scores
    static ref SCORES_CACHE: Mutex<HashMap<String, Arc<Mutex<Vec<Score>>>>> = {
        let mut list:HashMap<String, Arc<Mutex<Vec<Score>>>> = HashMap::new();
        let reader = open_database(SCORE_DATABASE_FILE);

        match reader {
            Err(e) => {
                println!("Error reading db: {:?}", e);
                Mutex::new(list)
            }
            Ok(mut reader) => {
                let mut count = reader.read_u128();

                while count > 0 {
                    count -= 1;
                    let score = Score::read(&mut reader);
                    let hash = score.beatmap_hash.to_owned();

                    if !list.contains_key(&hash.clone()) {
                        list.insert(hash.clone(), Arc::new(Mutex::new(Vec::new())));
                    }

                    let l = list.get_mut(&hash.clone()).unwrap();
                    l.lock().unwrap().push(score);
                }

                Mutex::new(list)
            }
        }
    };
}


pub fn save_score(s:Score) {
    let mut lock = SCORES_CACHE.lock().unwrap();
    if !lock.contains_key(&s.beatmap_hash) {
        lock.insert(s.beatmap_hash.clone(), Arc::new(Mutex::new(Vec::new())));
    }
    let x = lock.get(&s.beatmap_hash).unwrap();
    x.lock().unwrap().push(s);
}

pub fn get_scores(hash:String) -> Arc<Mutex<Vec<Score>>> {
    let mut lock = SCORES_CACHE.lock().unwrap();
    if !lock.contains_key(&hash) {
        lock.insert(hash.clone(), Arc::new(Mutex::new(Vec::new())));
    }
    lock.get(&hash).unwrap().clone()
}

pub fn save_all_scores() -> std::io::Result<()> {
    let mut writer = SerializationWriter::new();
    //TODO: make a better error handler lol
    let lock = SCORES_CACHE.lock().expect("Error locking scores cache for saving");

    let mut count:u128 = 0;
    lock.iter().for_each(|(_hash, scores)| {
        count += scores.lock().unwrap().len() as u128;
    });
    
    // write everything
    writer.write(count);
    lock.iter().for_each(|(_hash, scores)| {
        let scores = scores.lock().unwrap();
        scores.iter().for_each(|score| {
            writer.write(score.clone());
        });
    });

    // write file
    return save_database(SCORE_DATABASE_FILE, writer);
}
