use std::{collections::HashMap, fs::read_dir, path::Path, sync::Arc};
use parking_lot::Mutex;
use crate::{DOWNLOADS_DIR, gameplay::Beatmap};

// ugh
type ArcMutexBeatmap = Arc<Mutex<Beatmap>>;

pub struct BeatmapManager {
    pub beatmaps: Vec<ArcMutexBeatmap>,
    pub beatmaps_by_hash: HashMap<String, ArcMutexBeatmap>,

    pub dirty: bool, // might be useful later
}
impl BeatmapManager {
    pub fn new() -> Self {
        Self {
            beatmaps: Vec::new(),
            beatmaps_by_hash: HashMap::new(),
            dirty: false
        }
    }

    pub fn check_dirty(&mut self) -> bool {
        if self.dirty {
            self.dirty = false;
            true
        } else {
            false
        }
    }

    // TODO: finish implementing this
    #[allow(dead_code)]
    pub async fn check_downloads(_self:Arc<Mutex<Self>>) {
        let mut files = Vec::new();
        for file in read_dir(DOWNLOADS_DIR).unwrap() {
            if let Ok(file) = file {
                files.push(file)
            }
        }
        
        if files.len() == 0 {return}

        _self.lock().dirty = true;
    }

    // adders
    pub fn check_folder(&mut self, dir:String) {

        if !Path::new(&dir).is_dir() {return}
        let dir_files = read_dir(dir).unwrap();

        for file in dir_files {
            let file = file.unwrap().path();
            let file = file.to_str().unwrap();

            if file.ends_with(".osu") {
                let map = Beatmap::load(file.to_owned());
                // if map.lock().metadata.mode as u8 > 1 {
                //     // println!("skipping {}, not a taiko map or convert", map.lock().metadata.version_string());
                //     // continue;
                // }
                self.add_beatmap(map);
            }
        }
    }

    pub fn add_beatmap(&mut self, beatmap:ArcMutexBeatmap) {
        // check if we already have this map
        let new_hash = beatmap.lock().hash.clone();
        if self.beatmaps_by_hash.contains_key(&new_hash) {return println!("map already added")}

        // dont have it, add it
        self.beatmaps_by_hash.insert(new_hash, beatmap.clone());
        self.beatmaps.push(beatmap.clone());
        self.dirty = true;
    }

    // getters
    pub fn all_by_sets(&self) -> Vec<Vec<ArcMutexBeatmap>> { // list of sets as (list of beatmaps in the set)
        let mut set_map = HashMap::new();

        for beatmap in self.beatmaps.iter() {
            let m = beatmap.lock().metadata.clone();
            let key = format!("{}-{}[{}]",m.artist,m.title,m.creator); // good enough for now
            if !set_map.contains_key(&key) {set_map.insert(key.clone(), Vec::new());}
            set_map.get_mut(&key).unwrap().push(beatmap.clone());
        }

        let mut sets = Vec::new();
        set_map.values().for_each(|e|sets.push(e.to_owned()));
        sets
    }

    pub fn get_by_hash(&self, hash:String) -> Option<ArcMutexBeatmap> {
        match self.beatmaps_by_hash.get(&hash) {
            Some(b) => Some(b.clone()),
            None => None
        }
    }
}
