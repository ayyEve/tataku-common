use std::{collections::HashMap, sync::Arc};
use parking_lot::Mutex;

use taiko_rs_common::serialization::*;
use taiko_rs_common::types::{Replay, Score};
use crate::{REPLAYS_DIR, SCORE_DATABASE_FILE};


lazy_static::lazy_static! {
    /// SCORES_CACHE.get(.osu_hash) = list of scores
    static ref SCORES_CACHE: Mutex<HashMap<String, Arc<Mutex<Vec<Score>>>>> = Mutex::new(HashMap::new());
}


pub fn get_scores(hash:String) -> Arc<Mutex<Vec<Score>>> {
    let mut lock = SCORES_CACHE.lock();
    if !lock.contains_key(&hash) {
        lock.insert(hash.clone(), Arc::new(Mutex::new(Vec::new())));
    }
    lock.get(&hash).unwrap().clone()
}

pub fn save_score(s:&Score) {
    let mut lock = SCORES_CACHE.lock();
    if !lock.contains_key(&s.beatmap_hash) {
        lock.insert(s.beatmap_hash.clone(), Arc::new(Mutex::new(Vec::new())));
    }
    let x = lock.get(&s.beatmap_hash).unwrap();
    x.lock().push(s.clone());
}

pub fn save_all_scores() -> std::io::Result<()> {
    let mut writer = SerializationWriter::new();
    //TODO: make a better error handler lol
    let lock = SCORES_CACHE.lock();

    let mut count:u128 = 0;
    lock.iter().for_each(|(_hash, scores)| {
        count += scores.lock().len() as u128;
    });
    
    // write everything
    writer.write(count);
    lock.iter().for_each(|(_hash, scores)| {
        let scores = scores.lock();
        scores.iter().for_each(|score| {
            writer.write(score.clone());
        });
    });

    // write file
    return save_database(SCORE_DATABASE_FILE, writer);
}

pub fn save_replay(r:&Replay, s:&Score) -> std::io::Result<()> {
    let mut writer = SerializationWriter::new();
    writer.write(r.clone());

    let filename = format!("{}/{}.rs_replay", REPLAYS_DIR,s.hash());
    save_database(&filename, writer)
}

pub fn get_local_replay(score_hash:String) -> std::io::Result<Replay> {
    let fullpath = format!("{}/{}.rs_replay", REPLAYS_DIR, score_hash);
    let mut reader = open_database(&fullpath)?;
    Ok(reader.read())
}