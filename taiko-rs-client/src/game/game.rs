use std::sync::Arc;
use std::path::Path;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use tokio::runtime::{Builder, Runtime};
use glfw_window::GlfwWindow as AppWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::{Window, input::*, event_loop::*, window::WindowSettings};

// use crate::gameplay::Beatmap;
use crate::gameplay::{Beatmap, IngameManager};
use crate::{WINDOW_SIZE, SONGS_DIR, Vector2, menu::*};
use crate::render::{Color, Image, Rectangle, Renderable};
use crate::databases::{save_all_scores, save_replay, save_score};
use taiko_rs_common::types::{Replay, SpectatorFrames, UserAction};
use crate::helpers::{FpsDisplay, BenchmarkHelper, BeatmapManager, VolumeControl};
use crate::game::{InputManager, Settings, online::{USER_ITEM_SIZE, OnlineManager}};

/// background color
const GFX_CLEAR_COLOR:Color = Color::WHITE;
/// how long do transitions between gamemodes last?
const TRANSITION_TIME:u64 = 500;

pub struct Game {
    // engine things
    render_queue: Vec<Box<dyn Renderable>>,
    pub window: AppWindow,
    pub graphics: GlGraphics,
    pub input_manager: InputManager,
    pub online_manager: Arc<tokio::sync::Mutex<OnlineManager>>,
    pub threading: Runtime,
    
    pub menus: HashMap<&'static str, Arc<Mutex<dyn Menu<Game>>>>,
    pub beatmap_manager: Arc<Mutex<BeatmapManager>>, // must be thread safe
    
    pub current_mode: GameMode,
    pub queued_mode: GameMode,

    pub volume_controller: VolumeControl,

    // fps
    fps_display: FpsDisplay,
    update_display: FpsDisplay,
    input_update_display: FpsDisplay,

    // transition
    transition: Option<GameMode>,
    transition_last: Option<GameMode>,
    transition_timer: u64,

    // user list
    show_user_list: bool,

    // misc
    pub game_start: Instant,
    pub background_image: Option<Image>,
    // register_timings: (f32,f32,f32)
}
impl Game {
    pub fn new() -> Game {
        let mut game_init_benchmark = BenchmarkHelper::new("game::new");

        let opengl = OpenGL::V3_2;
        let window: AppWindow = WindowSettings::new("Taiko", [WINDOW_SIZE.x, WINDOW_SIZE.y])
            .graphics_api(opengl)
            .resizable(false)
            .build()
            .expect("Error creating window");
        game_init_benchmark.log("window created", true);

        let graphics = GlGraphics::new(opengl);
        game_init_benchmark.log("graphics created", true);

        let input_manager = InputManager::new();
        game_init_benchmark.log("input manager created", true);

        let online_manager = Arc::new(tokio::sync::Mutex::new(OnlineManager::new()));
        game_init_benchmark.log("online manager created", true);

        let threading = Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        game_init_benchmark.log("threading created", true);

        let mut g = Game {
            // engine
            window,
            graphics,
            threading,
            input_manager,
            online_manager,
            volume_controller:VolumeControl::new(),
            render_queue: Vec::new(),
            background_image: None,

            menus: HashMap::new(),
            beatmap_manager: Arc::new(Mutex::new(BeatmapManager::new())),
            current_mode: GameMode::None,
            queued_mode: GameMode::None,


            // fps
            fps_display: FpsDisplay::new("fps", 0),
            update_display: FpsDisplay::new("updates/s", 1),
            input_update_display: FpsDisplay::new("inputs/s", 2),

            // transition
            transition: None,
            transition_last: None,
            transition_timer: 0,

            // misc
            show_user_list: false,
            game_start: Instant::now(),
            // register_timings: (0.0,0.0,0.0)
        };
        game_init_benchmark.log("game created", true);

        g.init();
        game_init_benchmark.log("game initialized", true);

        g
    }

    pub fn init(&mut self) {
        let clone = self.online_manager.clone();
        self.threading.spawn(async move {
            loop {
                OnlineManager::start(clone.clone()).await;
                tokio::time::sleep(Duration::from_millis(1_000)).await;
            }
        });
        
        let mut loading_menu = LoadingMenu::new(self.beatmap_manager.clone());
        loading_menu.load(self);

        //region == menu setup ==
        let mut menu_init_benchmark = BenchmarkHelper::new("game::new");
        // main menu
        let main_menu = Arc::new(Mutex::new(MainMenu::new()));
        self.menus.insert("main", main_menu.clone());
        menu_init_benchmark.log("main menu created", true);

        // setup beatmap select menu
        let beatmap_menu = Arc::new(Mutex::new(BeatmapSelectMenu::new(self.beatmap_manager.clone())));
        self.menus.insert("beatmap", beatmap_menu.clone());
        menu_init_benchmark.log("beatmap menu created", true);

        // setup settings menu
        let settings_menu = Arc::new(Mutex::new(SettingsMenu::new()));
        self.menus.insert("settings", settings_menu.clone());
        menu_init_benchmark.log("settings menu created", true);


        self.queue_mode_change(GameMode::InMenu(Arc::new(Mutex::new(loading_menu))));
    }
    pub fn game_loop(mut self) {
        // input and rendering thread
        let mut events = Events::new(EventSettings::new());

        {
            let settings = Settings::get();
            if settings.unlimited_fps || cfg!(feature="unlimited_fps") {
                events.set_max_fps(10_000);
                events.set_ups(10_000);
            } else {
                events.set_max_fps(settings.fps_target);
                events.set_ups(settings.update_target);
            }
        }

        while let Some(e) = events.next(&mut self.window) {
            self.input_manager.handle_events(e.clone());
            if let Some(args) = e.update_args() {self.update(args.dt*1000.0)}
            if let Some(args) = e.render_args() {self.render(args)}
            if let Some(Button::Keyboard(_)) = e.press_args() {self.input_update_display.increment()}
            // e.resize(|args| println!("Resized '{}, {}'", args.window_size[0], args.window_size[1]));
        }
    }

    fn update(&mut self, _delta:f64) {
        self.update_display.increment();
        let mut current_mode = self.current_mode.clone();
        let elapsed = self.game_start.elapsed().as_millis() as u64;

        // read input events
        let mouse_pos = self.input_manager.mouse_pos;
        let mut mouse_buttons = self.input_manager.get_mouse_buttons();
        let mouse_moved = self.input_manager.get_mouse_moved();
        let mut scroll_delta = self.input_manager.get_scroll_delta();
        let mut keys = self.input_manager.all_down_once();
        let mods = self.input_manager.get_key_mods();
        let text = self.input_manager.get_text();
        let window_focus_changed = self.input_manager.get_changed_focus();

        // if keys.len() > 0 {
        //     self.register_timings = self.input_manager.get_register_delay();
        //     println!("register times: min:{}, max: {}, avg:{}", self.register_timings.0,self.register_timings.1,self.register_timings.2);
        // }

        // check for volume change
        if mouse_moved {self.volume_controller.on_mouse_move(mouse_pos)}
        if scroll_delta != 0.0 && self.volume_controller.on_mouse_wheel(scroll_delta, mods) {scroll_delta = 0.0}
        self.volume_controller.on_key_press(&mut keys, mods);
        
        // users list
        if keys.contains(&Key::F8) {
            self.show_user_list = !self.show_user_list;
            println!("show user list: {}", self.show_user_list);
        }
        if self.show_user_list {
            if let Ok(om) = self.online_manager.try_lock() {
                for (_, user) in &om.users {
                    if let Ok(mut u) = user.try_lock() {
                        if mouse_moved {u.on_mouse_move(mouse_pos)}
                        mouse_buttons.retain(|button| !u.on_click(mouse_pos, button.clone(), mods));
                    }
                }
            }
        }


        // run update on current move
        match current_mode {
            GameMode::Ingame(ref manager) => {
                // let settings = Settings::get();
                let mut lock = manager.lock();
                
                if keys.contains(&Key::Space) {lock.gamemode.lock().skip_intro()}
                // pause button, or focus lost
                if matches!(window_focus_changed, Some(false)) || keys.contains(&Key::Escape) {
                    lock.pause();
                    let menu = PauseMenu::new(manager.clone());
                    self.queue_mode_change(GameMode::InMenu(Arc::new(Mutex::new(menu))));
                }

                // offset adjust
                if keys.contains(&Key::Equals) {lock.increment_offset(5)}
                if keys.contains(&Key::Minus) {lock.increment_offset(-5)}

                for k in keys.iter() {
                    lock.key_down(*k);
                }

                // update, then check if complete
                lock.update();
                if lock.completed {
                    println!("beatmap complete");

                    let score = lock.score.as_ref().unwrap();
                    let replay = lock.replay.as_ref().unwrap();
                    println!("score + replay unwrapped");

                    // save score
                    save_score(&score);
                    println!("score saved");
                    match save_replay(&replay, &score) {
                        Ok(_)=> println!("replay saved ok"),
                        Err(e) => println!("error saving replay: {}", e),
                    }
                    println!("replay saved");
                    match save_all_scores() {
                        Ok(_) => println!("Scores saved successfully"),
                        Err(e) => println!("Failed to save scores! {}", e),
                    }
                    println!("all scores saved");

                    // submit score
                    #[cfg(feature = "online_scores")] 
                    {
                        self.threading.spawn(async move {
                            //TODO: do this async
                            println!("submitting score");
                            let mut writer = taiko_rs_common::serialization::SerializationWriter::new();
                            writer.write(score.clone());
                            writer.write(replay.clone());
                            let data = writer.data();
                            
                            let c = reqwest::Client::new();
                            let res = c.post("http://localhost:8000/score_submit")
                                .body(data)
                                .send().await;

                            match res {
                                Ok(_isgood) => {
                                    //TODO: do something with the response?
                                    println!("score submitted successfully");
                                },
                                Err(e) => println!("error submitting score: {}", e),
                            }
                        });
                    }

                    // show score menu
                    let menu = ScoreMenu::new(&score, lock.beatmap.clone());
                    self.queue_mode_change(GameMode::InMenu(Arc::new(Mutex::new(menu))));
                    println!("new menu set");

                    // cleanup memory-hogs in the beatmap object
                    // lock.cleanup();
                }
            }

            GameMode::Replaying(ref beatmap, ref replay, ref mut frame_index) => {
                // let mut beatmap = beatmap.lock();
                // let time = beatmap.time();
                // // read any frames that need to be read
                // loop {
                //     if *frame_index as usize >= replay.presses.len() {break}
                    
                //     let (press_time, pressed) = replay.presses[*frame_index as usize];
                //     if press_time > time {break}

                //     beatmap.hit(pressed);
                //     match pressed {
                //         KeyPress::LeftKat => {
                //             let mut hit = HalfCircle::new(
                //                 Color::BLUE,
                //                 HIT_POSITION,
                //                 1.0,
                //                 HIT_AREA_RADIUS,
                //                 true
                //             );
                //             hit.set_lifetime(DRUM_LIFETIME_TIME);
                //             self.render_queue.push(Box::new(hit));
                //         },
                //         KeyPress::LeftDon => {
                //             let mut hit = HalfCircle::new(
                //                 Color::RED,
                //                 HIT_POSITION,
                //                 1.0,
                //                 HIT_AREA_RADIUS,
                //                 true
                //             );
                //             hit.set_lifetime(DRUM_LIFETIME_TIME);
                //             self.render_queue.push(Box::new(hit));
                //         },
                //         KeyPress::RightDon => {
                //             let mut hit = HalfCircle::new(
                //                 Color::RED,
                //                 HIT_POSITION,
                //                 1.0,
                //                 HIT_AREA_RADIUS,
                //                 false
                //             );
                //             hit.set_lifetime(DRUM_LIFETIME_TIME);
                //             self.render_queue.push(Box::new(hit));
                //         },
                //         KeyPress::RightKat => {
                //             let mut hit = HalfCircle::new(
                //                 Color::BLUE,
                //                 HIT_POSITION,
                //                 1.0,
                //                 HIT_AREA_RADIUS,
                //                 false
                //             );
                //             hit.set_lifetime(DRUM_LIFETIME_TIME);
                //             self.render_queue.push(Box::new(hit));
                //         },
                //     }

                //     *frame_index += 1;
                // }
                
                // // pause button, or focus lost
                // if keys.contains(&Key::Escape) {
                //     beatmap.reset();
                //     let menu = self.menus.get("beatmap").unwrap().clone();
                //     self.queue_mode_change(GameMode::InMenu(menu));
                // }

                // // offset adjust
                // if keys.contains(&Key::Equals) {beatmap.increment_offset(5)}
                // if keys.contains(&Key::Minus) {beatmap.increment_offset(-5)}

                // beatmap.update();

                // if beatmap.completed {
                //     // show score menu
                //     let menu = ScoreMenu::new(&beatmap.score.as_ref().unwrap(), Arc::new(Mutex::new(beatmap.clone())));
                //     self.queue_mode_change(GameMode::InMenu(Arc::new(Mutex::new(menu))));

                //     beatmap.cleanup();
                // }
            
            }
            
            GameMode::InMenu(ref menu) => {
                let mut menu = menu.lock();

                // menu input events

                // clicks
                for b in mouse_buttons { 
                    // game.start_map() can happen here, which needs &mut self
                    menu.on_click(mouse_pos, b, mods, self);
                }
                // mouse move
                if mouse_moved {menu.on_mouse_move(mouse_pos, self)}
                // mouse scroll
                if scroll_delta.abs() > 0.0 {menu.on_scroll(scroll_delta, self)}

                //check keys down
                for key in keys {menu.on_key_press(key, self, mods)}
                //TODO: check keys up (or remove it, probably not used anywhere)

                // check text
                if text.len() > 0 {menu.on_text(text);}

                // window focus change
                if let Some(has_focus) = window_focus_changed {
                    menu.on_focus_change(has_focus, self);
                }

                menu.update(self);
            }

            GameMode::Spectating(ref data, ref state, ref _beatmap) => {
                let mut data = data.lock();

                // (try to) read pending data from the online manager
                match self.online_manager.try_lock() {
                    Ok(mut online_manager) => data.extend(online_manager.get_pending_spec_frames()),
                    Err(e) => println!("hmm, {}", e),
                }

                match &state {
                    SpectatorState::Buffering => {},
                    SpectatorState::Watching => todo!(),
                    SpectatorState::Paused => todo!(),
                    SpectatorState::MapChanging => todo!(),
                }
            }

            GameMode::None => {
                // might be transitioning
                if self.transition.is_some().clone() && elapsed - self.transition_timer > TRANSITION_TIME / 2 {
                    let trans = self.transition.as_ref().unwrap().clone();
                    self.queue_mode_change(trans);
                    self.transition = None;
                    self.transition_timer = elapsed;
                }
            }

            _ => {}
        }
        
        // update game mode
        match &self.queued_mode {
            // queued mode didnt change, set the unlocked's mode to the updated mode
            GameMode::None => self.current_mode = current_mode,
            GameMode::Closing => {
                Settings::get().save();
                self.window.set_should_close(true);
            }

            _ => {
                // if the mode is being changed, clear all shapes, even ones with a lifetime
                self.clear_render_queue(true);

                let online_manager = self.online_manager.clone();

                // let cloned_mode = self.queued_mode.clone();
                // self.threading.spawn(async move {
                //     online_manager.lock().await.discord.change_status(cloned_mode);
                //     OnlineManager::set_action(online_manager, UserAction::Leaving, String::new()).await;
                // });

                match &self.queued_mode {
                    GameMode::Ingame(map) => {
                        let (m, h) = { 
                            let lock = map.lock();

                            (lock.beatmap.metadata.clone(), lock.beatmap.hash.clone())
                        };

                        let text = format!("{}-{}[{}]\n{}",m.artist,m.title,m.version,h);

                        if let Ok(t) = opengl_graphics::Texture::from_path(m.image_filename.clone(), &opengl_graphics::TextureSettings::new()) {
                            self.background_image = Some(Image::new(Vector2::zero(), f64::MAX, t, WINDOW_SIZE));
                        } else {
                            self.background_image = None;
                        }

                        self.threading.spawn(async move {
                            OnlineManager::set_action(online_manager, UserAction::Ingame, text).await;
                        });
                    },
                    GameMode::InMenu(_) => {
                        self.threading.spawn(async move {
                            OnlineManager::set_action(online_manager, UserAction::Idle, String::new()).await;
                        });
                    },
                    GameMode::Closing => {
                        // send logoff
                        self.threading.spawn(async move {
                            OnlineManager::set_action(online_manager, UserAction::Leaving, String::new()).await;
                        });
                    }
                    _ => {}
                }

                let mut do_transition = true;
                match &self.current_mode {
                    GameMode::None => do_transition = false,
                    GameMode::InMenu(menu) if menu.lock().get_name() == "pause" => do_transition = false,
                    _ => {}
                }

                if do_transition {
                    // do a transition
                    let qm = &self.queued_mode;
                    self.transition = Some(qm.clone());
                    self.transition_timer = elapsed;
                    self.transition_last = Some(self.current_mode.clone());
                    self.queued_mode = GameMode::None;
                    self.current_mode = GameMode::None;
                } else {
                    // old mode was none, or was pause menu, transition to new mode
                    let mode = self.queued_mode.clone();

                    self.current_mode = mode.clone();
                    self.queued_mode = GameMode::None;


                    if let GameMode::InMenu(menu) = &self.current_mode {
                        menu.lock().on_change(true);
                    }
                }
            }
        }
    }

    fn render(&mut self, args: RenderArgs) {
        let mut renderables: Vec<Box<dyn Renderable>> = Vec::new();
        let settings = Settings::get();
        let elapsed = self.game_start.elapsed().as_millis() as u64;

        // draw background image here
        if let Some(img) = self.background_image.as_ref() {
            // dim
            renderables.push(Box::new(img.clone()));
            // println!("{} > {}", img.get_depth(), f64::MAX - 1.0);
        }
        let mut color = Color::BLACK;
        color.a = settings.background_dim;
        renderables.push(Box::new(Rectangle::new(
            color,
            f64::MAX - 1.0,
            Vector2::zero(),
            WINDOW_SIZE,
            None
        )));

        // mode
        match &self.current_mode {
            GameMode::Ingame(manager) => manager.lock().draw(args, &mut renderables),
            GameMode::InMenu(menu) => renderables.extend(menu.lock().draw(args)),
            // GameMode::Replaying(beatmap,_,_) => {
            //     renderables.extend(beatmap.lock().draw(args));

            //     // draw watching X text
            // }
            
            _ => {}
        }

        // transition
        if self.transition_timer > 0 && elapsed - self.transition_timer < TRANSITION_TIME {
            // probably transitioning

            // draw fade in rect
            let diff = elapsed as f64 - self.transition_timer as f64;

            let mut alpha = diff / (TRANSITION_TIME as f64 / 2.0);
            if self.transition.is_none() {alpha = 1.0 - diff / TRANSITION_TIME as f64}

            renderables.push(Box::new(Rectangle::new(
                [0.0, 0.0, 0.0, alpha as f32].into(),
                -f64::MAX,
                Vector2::zero(),
                WINDOW_SIZE,
                None
            )));

            // draw old mode
            match (&self.current_mode, &self.transition_last) {
                (GameMode::None, Some(GameMode::InMenu(menu))) => renderables.extend(menu.lock().draw(args)),
                _ => {}
            }
        }

        // users list
        if self.show_user_list {

            //TODO: move the set_pos code to update or smth
            let mut counter = 0;
            
            if let Ok(om) = self.online_manager.try_lock() {
                for (_, user) in &om.users.clone() {
                    if let Ok(mut u) = user.try_lock() {
                        let x = if counter % 2 == 0 {0.0} else {USER_ITEM_SIZE.x};
                        let y = (counter - counter % 2) as f64 * USER_ITEM_SIZE.y;
                        u.set_pos(Vector2::new(x,y));

                        counter += 1;
                        renderables.extend(u.draw(args, Vector2::zero(), -100.0));
                    }
                }
            }
        }

        // volume control
        self.render_queue.extend(self.volume_controller.draw());

        // add the things we just made to the render queue
        self.render_queue.extend(renderables);


        // draw fps's
        self.render_queue.push(Box::new(self.fps_display.draw()));
        self.render_queue.push(Box::new(self.update_display.draw()));
        self.render_queue.push(Box::new(self.input_update_display.draw()));

        // sort the queue here (so it only needs to be sorted once per frame, instead of every time a shape is added)
        self.render_queue.sort_by(|a, b| b.get_depth().partial_cmp(&a.get_depth()).unwrap());

        // slice the queue because loops
        let queue = self.render_queue.as_mut_slice();
        self.graphics.draw(args.viewport(), |c, g| {
            graphics::clear(GFX_CLEAR_COLOR.into(), g);

            for i in queue.as_mut() {
                if i.get_spawn_time() == 0 {i.set_spawn_time(elapsed);}
                i.draw(g, c);
            }
        });
        
        self.clear_render_queue(false);
        self.fps_display.increment();
    }

    pub fn clear_render_queue(&mut self, remove_all:bool) {
        if remove_all {return self.render_queue.clear()}

        let elapsed = self.game_start.elapsed().as_millis() as u64;
        // only return items who's lifetime has expired
        self.render_queue.retain(|e| {
            let lifetime = e.get_lifetime();
            lifetime > 0 && elapsed - e.get_spawn_time() < lifetime
        });
    }
    
    pub fn queue_mode_change(&mut self, mode:GameMode) {self.queued_mode = mode}

    /// shortcut for setting the game's background texture to a beatmap's image
    pub fn set_background_beatmap(&mut self, beatmap:Arc<Mutex<Beatmap>>) {
        match opengl_graphics::Texture::from_path(beatmap.lock().metadata.image_filename.clone(), &opengl_graphics::TextureSettings::new()) {
            Ok(tex) => self.background_image = Some(Image::new(Vector2::zero(), f64::MAX, tex, WINDOW_SIZE)),
            Err(e) => {
                println!("Error loading beatmap texture: {}", e);
                self.background_image = None; //TODO!: use a known good background image
            },
        }
    }

    /// extract all zips from the downloads folder into the songs folder. not a static function as it uses threading
    pub fn extract_all(&self) {
        let runtime = &self.threading;

        // check for new maps
        if let Ok(files) = std::fs::read_dir(crate::DOWNLOADS_DIR) {
            for file in files {
                if let Ok(filename) = file {
                    runtime.spawn(async move {

                        // unzip file into ./Songs

                        while let Err(_) = std::fs::File::open(filename.path().to_str().unwrap()) {
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
                                println!("File {} extracted to \"{}\"", i, outpath.display());
                                std::fs::create_dir_all(&outpath).unwrap();
                            } else {
                                println!("File {} extracted to \"{}\" ({} bytes)", i, outpath.display(), file.size());
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
                            Err(e) => println!("error deleting file: {}", e),
                        }
                    });
                }
            }
        }
    }
}


#[derive(Clone)]
pub enum GameMode {
    None, // use this as the inital game mode, but me sure to change it after
    Closing,
    Ingame(Arc<Mutex<IngameManager>>),
    InMenu(Arc<Mutex<dyn Menu<Game>>>),
    Replaying(Arc<Mutex<Beatmap>>, Replay, u64),

    #[allow(dead_code)]
    Spectating(Arc<Mutex<SpectatorFrames>>, SpectatorState, Option<Arc<Mutex<Beatmap>>>), // frames awaiting replay, state, beatmap
    // Multiplaying(MultiplayerState), // wink wink nudge nudge (dont hold your breath)
}


#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum SpectatorState {
    Buffering, // waiting for data
    Watching, // host playing
    Paused, // host paused
    MapChanging, // host is changing map
}