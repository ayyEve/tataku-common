use std::{collections::HashMap, fs::read_dir, path::Path, sync::Arc};
use parking_lot::Mutex;
use rand::Rng;
use crate::{DOWNLOADS_DIR, game::{Audio, Game}, gameplay::Beatmap};

// ugh
type ArcMutexBeatmap = Arc<Mutex<Beatmap>>;

pub struct BeatmapManager {
    pub current_beatmap: Option<ArcMutexBeatmap>,
    pub beatmaps: Vec<ArcMutexBeatmap>,
    pub beatmaps_by_hash: HashMap<String, ArcMutexBeatmap>,

    pub dirty: bool, // might be useful later

    /// previously played maps
    played: Vec<String>,
    /// current index of previously played maps
    play_index: usize
}
impl BeatmapManager {
    pub fn new() -> Self {
        Self {
            current_beatmap: None,
            beatmaps: Vec::new(),
            beatmaps_by_hash: HashMap::new(),
            dirty: false,

            played: Vec::new(),
            play_index: 0
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


    pub fn set_current_beatmap(&mut self, game:&mut Game, beatmap: ArcMutexBeatmap) {
        if let Some(map) = self.current_beatmap.clone() {
            self.played.push(map.lock().hash.clone());
        }

        // play song
        let audio_filename = beatmap.lock().metadata.audio_filename.clone();
        Audio::play_song(audio_filename, false); // restart doesnt matter as this should be the first song to play

        // set bg
        game.set_background_beatmap(beatmap);
        
        //TODO! somehow select the map in beatmap select?
        // might be best to have a current_beatmap value in beatmap_manager
    }
    
    
    pub fn random_beatmap(&self) -> Option<ArcMutexBeatmap> {
        if self.beatmaps.len() > 0 {
            let ind = rand::thread_rng().gen_range(0..self.beatmaps.len());
            let map = self.beatmaps[ind].clone();
            Some(map.clone())
        } else {
            None
        }
    }

    pub fn next_beatmap(&mut self) -> Option<ArcMutexBeatmap> {
        self.play_index += 1;

        if self.play_index < self.played.len() {
            let hash = self.played[self.play_index].clone();
            self.get_by_hash(hash).clone()
        } else {
            let map = self.random_beatmap();
            if let Some(map) = map.clone() {
                self.played.push(map.lock().hash.clone());
            }
            map
        }
    }
    pub fn previous_beatmap(&mut self) -> Option<ArcMutexBeatmap> {
        if self.play_index == 0 {
            return None
        }
        self.play_index -= 1;
        
        match self.played.get(self.play_index) {
            Some(hash) => self.get_by_hash(hash.clone()).clone(),
            None => None
        }
    }

}
