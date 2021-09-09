use std::{collections::HashMap, fs::{DirEntry, read_dir}, path::Path, sync::Arc, time::Duration};

use rand::Rng;
use parking_lot::Mutex;
use crate::{DOWNLOADS_DIR, SONGS_DIR, game::{Audio, Game}, gameplay::{Beatmap, BeatmapMeta}, get_file_hash};

const DOWNLOAD_CHECK_INTERVAL:u64 = 10_000;

lazy_static::lazy_static! {
    pub static ref BEATMAP_MANAGER: Arc<Mutex<BeatmapManager>> = Arc::new(Mutex::new(BeatmapManager::new()));
}

pub struct BeatmapManager {
    pub initialized: bool,

    pub current_beatmap: Option<BeatmapMeta>,
    pub beatmaps: Vec<BeatmapMeta>,
    pub beatmaps_by_hash: HashMap<String, BeatmapMeta>,

    /// previously played maps
    played: Vec<String>,
    /// current index of previously played maps
    play_index: usize,

    new_maps: Vec<BeatmapMeta>
}
impl BeatmapManager {
    pub fn new() -> Self {
        Self {
            initialized: false,

            current_beatmap: None,
            beatmaps: Vec::new(),
            beatmaps_by_hash: HashMap::new(),

            played: Vec::new(),
            play_index: 0,
            new_maps: Vec::new(),
        }
    }

    // download checking
    pub fn get_new_maps(&mut self) -> Vec<BeatmapMeta> {
        std::mem::take(&mut self.new_maps)
    }
    fn check_downloads(runtime:&tokio::runtime::Runtime) {
        if read_dir(DOWNLOADS_DIR).unwrap().count() > 0 {
            extract_all(runtime);

            let mut folders = Vec::new();
            read_dir(SONGS_DIR)
                .unwrap()
                .for_each(|f| {
                    let f = f.unwrap().path();
                    folders.push(f.to_str().unwrap().to_owned());
                });

            for f in folders {BEATMAP_MANAGER.lock().check_folder(f)}
        }

    }
    pub fn download_check_loop(game:&Game) {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        game.threading.spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(DOWNLOAD_CHECK_INTERVAL)).await;
                BeatmapManager::check_downloads(&runtime);
            }
        });
    }

    // adders
    pub fn check_folder(&mut self, dir:String) {
        if !Path::new(&dir).is_dir() {return}
        let dir_files = read_dir(dir).unwrap();

        for file in dir_files {
            let file = file.unwrap().path();
            let file = file.to_str().unwrap();

            if file.ends_with(".osu") {

                // check file paths first
                for i in self.beatmaps.iter() {
                    if i.file_path == file {
                        continue;
                    }
                }

                match get_file_hash(file) {
                    Ok(hash) => if self.beatmaps_by_hash.contains_key(&hash) {continue},
                    Err(e) => {
                        println!("error getting hash for file {}: {}", file, e);
                        continue;
                    }
                }

                let map = Beatmap::load(file.to_owned()).metadata;
                self.add_beatmap(&map);


                // if it got here, it shouldnt be in the database
                // so we should add it
                {
                    let lock = crate::databases::DATABASE.lock();
                    let res = lock.prepare(&insert_metadata(&map)).unwrap().execute([]);
                    if let Err(e) = res {
                        println!("error inserting metadata: {}", e);
                    }
                }
            }
        }
    }

    pub fn add_beatmap(&mut self, beatmap:&BeatmapMeta) {
        // check if we already have this map
        let new_hash = beatmap.beatmap_hash.clone();
        if self.beatmaps_by_hash.contains_key(&new_hash) {return println!("map already added")}

        // dont have it, add it
        if self.initialized {self.new_maps.push(beatmap.clone())}
        self.beatmaps_by_hash.insert(new_hash, beatmap.clone());
        self.beatmaps.push(beatmap.clone());
    }

    // setters
    pub fn set_current_beatmap(&mut self, game:&mut Game, beatmap:&BeatmapMeta, do_async:bool, use_preview_time:bool) {
        self.current_beatmap = Some(beatmap.clone());
        if let Some(map) = self.current_beatmap.clone() {
            self.played.push(map.beatmap_hash.clone());
        }

        // play song
        let audio_filename = beatmap.audio_filename.clone();
        let time = if use_preview_time {beatmap.audio_preview} else {0.0};
        if do_async {
            game.threading.spawn(async move {
                Audio::play_song(audio_filename, false, time);
            });
        } else {
            Audio::play_song(audio_filename, false, time);
        }

        // set bg
        game.set_background_beatmap(beatmap);
        
        //TODO! somehow select the map in beatmap select?
        // might be best to have a current_beatmap value in beatmap_manager
    }
    

    // getters
    pub fn all_by_sets(&self) -> Vec<Vec<BeatmapMeta>> { // list of sets as (list of beatmaps in the set)
        let mut set_map = HashMap::new();

        for beatmap in self.beatmaps.iter() {
            let m = beatmap.clone();
            let key = format!("{}-{}[{}]", m.artist, m.title, m.creator); // good enough for now
            if !set_map.contains_key(&key) {set_map.insert(key.clone(), Vec::new());}
            set_map.get_mut(&key).unwrap().push(beatmap.clone());
        }

        let mut sets = Vec::new();
        set_map.values().for_each(|e|sets.push(e.to_owned()));
        sets
    }
    pub fn get_by_hash(&self, hash:&String) -> Option<BeatmapMeta> {
        match self.beatmaps_by_hash.get(hash) {
            Some(b) => Some(b.clone()),
            None => None
        }
    }

    pub fn random_beatmap(&self) -> Option<BeatmapMeta> {
        if self.beatmaps.len() > 0 {
            let ind = rand::thread_rng().gen_range(0..self.beatmaps.len());
            let map = self.beatmaps[ind].clone();
            Some(map.clone())
        } else {
            None
        }
    }

    pub fn next_beatmap(&mut self) -> Option<BeatmapMeta> {
        self.play_index += 1;
        if self.play_index < self.played.len() {
            let hash = self.played[self.play_index].clone();
            self.get_by_hash(&hash).clone()
        } else {
            self.random_beatmap()
        }
    }
    pub fn previous_beatmap(&mut self) -> Option<BeatmapMeta> {
        if self.play_index == 0 {return None}
        self.play_index -= 1;
        
        match self.played.get(self.play_index) {
            Some(hash) => self.get_by_hash(&hash).clone(),
            None => None
        }
    }

}


fn insert_metadata(map: &BeatmapMeta) -> String {
    format!("INSERT INTO beatmaps (
        beatmap_path, beatmap_hash,

        playmode, beatmap_version,
        artist, artist_unicode,
        title, title_unicode,
        creator, version,

        audio_filename, image_filename,
        audio_preview, duration,
        
        hp, od, cs, ar,
        
        slider_multiplier, slider_tick_rate
    ) VALUES (
        \"{}\", \"{}\",

        {}, {}, 
        \"{}\", \"{}\",
        \"{}\", \"{}\",
        \"{}\", \"{}\",

        \"{}\", \"{}\",
        {}, {},

        {}, {}, {}, {},

        {}, {}
    )",
    map.file_path, map.beatmap_hash, 

    map.mode as u8, map.beatmap_version,
    map.artist, map.artist_unicode,
    map.title, map.title_unicode,
    map.creator, map.version,
    
    map.audio_filename, map.image_filename,
    map.audio_preview, map.duration,

    map.hp, map.od, map.cs, map.ar,

    map.slider_multiplier, map.slider_tick_rate
    )
}



pub fn extract_all(runtime:&tokio::runtime::Runtime) {

    // check for new maps
    if let Ok(files) = std::fs::read_dir(crate::DOWNLOADS_DIR) {
        let completed = Arc::new(Mutex::new(0));

        let files:Vec<std::io::Result<DirEntry>> = files.collect();
        let len = files.len();
        println!("[extract] files: {:?}", files);

        for file in files {
            println!("[extract] looping file {:?}", file);
            let completed = completed.clone();

            match file {
                Ok(filename) => {
                    println!("[extract] file ok");
                    runtime.spawn(async move {
                        println!("[extract] reading file {:?}", filename);

                        let mut error_counter = 0;
                        // unzip file into ./Songs
                        while let Err(e) = std::fs::File::open(filename.path().to_str().unwrap()) {
                            println!("[extract] error opening osz file: {}", e);
                            error_counter += 1;

                            // if we've waited 5 seconds and its still broken
                            if error_counter > 5 {
                                println!("[extract] 5 errors opening osz file: {}", e);
                                return;
                            }

                            tokio::time::sleep(Duration::from_millis(1000)).await;
                        }

                        let file = std::fs::File::open(filename.path().to_str().unwrap()).unwrap();
                        let mut archive = zip::ZipArchive::new(file).unwrap();
                        
                        for i in 0..archive.len() {
                            let mut file = archive.by_index(i).unwrap();
                            let mut outpath = match file.enclosed_name() {
                                Some(path) => path,
                                None => continue,
                            };

                            let x = outpath.to_str().unwrap();
                            let y = format!("{}/{}/", SONGS_DIR, filename.file_name().to_str().unwrap().trim_end_matches(".osz"));
                            let z = &(y + x);
                            outpath = Path::new(z);

                            if (&*file.name()).ends_with('/') {
                                println!("[extract] File {} extracted to \"{}\"", i, outpath.display());
                                std::fs::create_dir_all(&outpath).unwrap();
                            } else {
                                println!("[extract] File {} extracted to \"{}\" ({} bytes)", i, outpath.display(), file.size());
                                if let Some(p) = outpath.parent() {
                                    if !p.exists() {std::fs::create_dir_all(&p).unwrap()}
                                }
                                let mut outfile = std::fs::File::create(&outpath).unwrap();
                                std::io::copy(&mut file, &mut outfile).unwrap();
                            }

                            // Get and Set permissions
                            // #[cfg(unix)] {
                            //     use std::os::unix::fs::PermissionsExt;
                            //     if let Some(mode) = file.unix_mode() {
                            //         fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
                            //     }
                            // }
                        }
                    
                        match std::fs::remove_file(filename.path().to_str().unwrap()) {
                            Ok(_) => {},
                            Err(e) => println!("[extract] error deleting file: {}", e),
                        }
                        
                        println!("[extract] done");
                        *completed.lock() += 1;
                    });
                }
                Err(e) => {
                    println!("error with file: {}", e);
                }
            }
        }
    
        
        while *completed.lock() < len {
            println!("waiting for downloads {} of {}", *completed.lock(), len);
            std::thread::sleep(Duration::from_millis(500));
        }
    }
}