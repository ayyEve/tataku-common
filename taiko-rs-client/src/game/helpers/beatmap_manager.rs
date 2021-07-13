use std::{collections::HashMap, fs::read_dir, path::Path, sync::{Arc, Mutex}};

use crate::{DOWNLOADS_DIR, gameplay::Beatmap};

pub struct BeatmapManager {
    pub beatmaps: Vec<Arc<Mutex<Beatmap>>>,

    pub dirty: bool, // might be useful later
}
impl BeatmapManager {
    pub fn new() -> Self {
        Self {
            beatmaps:Vec::new(),
            dirty: false
        }
    }

    // TODO: finish implementing this
    pub async fn check_downloads(_self:Arc<Mutex<Self>>) {
        let mut files = Vec::new();
        for file in read_dir(DOWNLOADS_DIR).unwrap() {
            if let Ok(file) = file{
                files.push(file)
            }
        }
        
        if files.len() == 0 {return}

        _self.lock().unwrap().dirty = true;
    }

    pub fn check_folder(&mut self, dir:String) {

        if !Path::new(&dir).is_dir() {return}
        let dir_files = read_dir(dir).unwrap();

        for file in dir_files {
            let file = file.unwrap().path();
            let file = file.to_str().unwrap();

            if file.ends_with(".osu") {
                let map = Beatmap::load(file.to_owned());
                if map.lock().unwrap().metadata.mode as u8 > 1 {
                    println!("skipping {}, not a taiko map or convert", map.lock().unwrap().metadata.version_string());
                    continue;
                }
                self.add_beatmap(map);
            }
        }
    }

    pub fn add_beatmap(&mut self, beatmap:Arc<Mutex<Beatmap>>) {

        // check if we already have this map
        let new_hash = beatmap.lock().unwrap().hash.clone();
        for i in self.beatmaps.iter() {if i.lock().unwrap().hash == new_hash {println!("skipping map"); return}}

        // dont have it, add it
        self.beatmaps.push(beatmap);
        self.dirty = true;
    }

    pub fn check_dirty(&mut self) -> bool {
        if self.dirty {
            self.dirty = false;
            true
        } else {
            false
        }
    }

    pub fn all_by_sets(&self) -> Vec<Vec<Arc<Mutex<Beatmap>>>> { // list of sets as (list of beatmaps in the set)
        let mut set_map = HashMap::new();

        for beatmap in self.beatmaps.iter() {
            let m = beatmap.lock().unwrap().metadata.clone();
            let key = format!("{}-{}[{}]",m.artist,m.title,m.creator); // good enough for now
            if !set_map.contains_key(&key) {set_map.insert(key.clone(), Vec::new());}
            set_map.get_mut(&key).unwrap().push(beatmap.clone());
        }

        let mut sets = Vec::new();
        set_map.values().for_each(|e|sets.push(e.to_owned()));

        println!("set len: {}, map len: {}", sets.len(), self.beatmaps.len());
        sets
    }
}
