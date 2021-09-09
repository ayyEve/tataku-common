use std::collections::HashMap;
use std::time::{Duration, Instant};

use ayyeve_piston_ui::render::Circle;
use tokio::runtime::{Builder, Runtime};
use glfw_window::GlfwWindow as AppWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::{Window, input::*, event_loop::*, window::WindowSettings};

use crate::databases::{save_replay, save_score};
use crate::render::{Color, Image, Rectangle, Renderable};
use taiko_rs_common::types::{SpectatorFrames, UserAction};
use crate::gameplay::{Beatmap, BeatmapMeta, IngameManager};
use crate::helpers::{FpsDisplay, BenchmarkHelper, VolumeControl};
use crate::{window_size, Vector2, DOWNLOADS_DIR, menu::*, sync::{Arc, Mutex}};
use crate::game::{Settings, audio::Audio, online::{USER_ITEM_SIZE, OnlineManager}, managers::{InputManager, BeatmapManager, NotificationManager, NOTIFICATION_MANAGER}};

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
    pub current_state: GameState,
    pub queued_state: GameState,

    pub volume_controller: VolumeControl,

    // fps
    fps_display: FpsDisplay,
    update_display: FpsDisplay,
    input_update_display: FpsDisplay,

    // transition
    transition: Option<GameState>,
    transition_last: Option<GameState>,
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
        let mut game_init_benchmark = BenchmarkHelper::new("Game::new");

        let opengl = OpenGL::V3_2;
        let mut window: AppWindow = WindowSettings::new("Taiko-rs", [window_size().x, window_size().y])
            .graphics_api(opengl)
            .resizable(false)
            .build()
            .expect("Error creating window");
        window.window.set_cursor_mode(glfw::CursorMode::Hidden);
        game_init_benchmark.log("window created", true);

        {
            //TODO: somehow make sure this file exists?
            match image::open("./icon-small.png") {
                Ok(img) => {
                    window.window.set_icon(vec![img.into_rgba8()]);
                    game_init_benchmark.log("window icon set", true);
                }
                Err(e) => {
                    game_init_benchmark.log(&format!("error setting window icon: {}", e), true);
                }
            }
        }
        

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
            current_state: GameState::None,
            queued_state: GameState::None,

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

        // online loop
        self.threading.spawn(async move {
            loop {
                OnlineManager::start(clone.clone()).await;
                tokio::time::sleep(Duration::from_millis(1_000)).await;
            }
        });

        // beatmap manager loop
        BeatmapManager::download_check_loop(self);
        
        
        let mut loading_menu = LoadingMenu::new();
        loading_menu.load(self);

        //region == menu setup ==
        let mut menu_init_benchmark = BenchmarkHelper::new("Game::init");
        // main menu
        let main_menu = Arc::new(Mutex::new(MainMenu::new()));
        self.menus.insert("main", main_menu.clone());
        menu_init_benchmark.log("main menu created", true);

        // setup beatmap select menu
        let beatmap_menu = Arc::new(Mutex::new(BeatmapSelectMenu::new()));
        self.menus.insert("beatmap", beatmap_menu.clone());
        menu_init_benchmark.log("beatmap menu created", true);

        // setup settings menu
        let settings_menu = Arc::new(Mutex::new(SettingsMenu::new()));
        self.menus.insert("settings", settings_menu.clone());
        menu_init_benchmark.log("settings menu created", true);

        self.queue_state_change(GameState::InMenu(Arc::new(Mutex::new(loading_menu))));
    }
    pub fn game_loop(mut self) {
        // input and rendering thread
        let mut events = Events::new(EventSettings::new());

        {
            let settings = Settings::get();
            events.set_max_fps(settings.fps_target);
            events.set_ups(settings.update_target);
        }

        while let Some(e) = events.next(&mut self.window) {
            self.input_manager.handle_events(e.clone());
            if let Some(args) = e.update_args() {self.update(args.dt*1000.0)}
            if let Some(args) = e.render_args() {self.render(args)}
            if let Some(Button::Keyboard(_)) = e.press_args() {self.input_update_display.increment()}


            if let Event::Input(Input::FileDrag(FileDrag::Drop(d)), _) = e {
                println!("got file: {:?}", d);
                let path = d.as_path();
                let filename = d.file_name();

                if let Some(ext) = d.extension() {
                    let ext = ext.to_str().unwrap();
                    match *&ext {
                        "osz" => {
                            if let Err(e) = std::fs::copy(path, format!("{}/{}", DOWNLOADS_DIR, filename.unwrap().to_str().unwrap())) {
                                println!("Error moving file: {}", e);
                                NotificationManager::add_text_notification(
                                    &format!("Error moving file\n{}", e), 
                                    2_000.0, 
                                    Color::RED
                                );
                            }
                        }

                        _ => {
                            NotificationManager::add_text_notification(
                                &format!("What is this?"), 
                                1_000.0, 
                                Color::RED
                            );
                        }
                    }
                }
            }
            // e.resize(|args| println!("Resized '{}, {}'", args.window_size[0], args.window_size[1]));
        }
    }

    fn update(&mut self, _delta:f64) {
        // let timer = Instant::now();
        self.update_display.increment();
        let current_state = self.current_state.clone();
        let elapsed = self.game_start.elapsed().as_millis() as u64;

        // read input events
        let mouse_pos = self.input_manager.mouse_pos;
        let mut mouse_down = self.input_manager.get_mouse_down();
        let mouse_up = self.input_manager.get_mouse_up();
        let mouse_moved = self.input_manager.get_mouse_moved();
        let mut scroll_delta = self.input_manager.get_scroll_delta();

        let mut keys_down = self.input_manager.get_keys_down();
        let keys_up = self.input_manager.get_keys_up();
        let mods = self.input_manager.get_key_mods();
        let text = self.input_manager.get_text();
        let window_focus_changed = self.input_manager.get_changed_focus();

        // if keys.len() > 0 {
        //     self.register_timings = self.input_manager.get_register_delay();
        //     println!("register times: min:{}, max: {}, avg:{}", self.register_timings.0,self.register_timings.1,self.register_timings.2);
        // }

        if mouse_down.len() > 0 {
            // check notifs
            if NOTIFICATION_MANAGER.lock().on_click(mouse_pos, self) {
                mouse_down.clear();
            }
        }

        // check for volume change
        if mouse_moved {self.volume_controller.on_mouse_move(mouse_pos)}
        if scroll_delta != 0.0 && self.volume_controller.on_mouse_wheel(scroll_delta, mods) {scroll_delta = 0.0}
        self.volume_controller.on_key_press(&mut keys_down, mods);
        
        // users list
        if keys_down.contains(&Key::F8) {
            self.show_user_list = !self.show_user_list;
            println!("Show user list: {}", self.show_user_list);
        }
        if self.show_user_list {
            if let Ok(om) = self.online_manager.try_lock() {
                for (_, user) in &om.users {
                    if let Ok(mut u) = user.try_lock() {
                        if mouse_moved {u.on_mouse_move(mouse_pos)}
                        mouse_down.retain(|button| !u.on_click(mouse_pos, button.clone(), mods));
                    }
                }
            }
        }


        // run update on current state
        match current_state {
            GameState::Ingame(ref manager) => {
                let mut lock = manager.lock();
                
                // pause button, or focus lost, only if not replaying
                if !lock.replaying && matches!(window_focus_changed, Some(false)) || keys_down.contains(&Key::Escape) {
                    lock.pause();
                    let menu = PauseMenu::new(manager.clone());
                    self.queue_state_change(GameState::InMenu(Arc::new(Mutex::new(menu))));
                }

                // offset adjust
                if keys_down.contains(&Key::Equals) {lock.increment_offset(5.0)}
                if keys_down.contains(&Key::Minus) {lock.increment_offset(-5.0)}

                if mouse_moved {
                    lock.mouse_move(mouse_pos);
                }
                for btn in mouse_down {
                    lock.mouse_down(btn);
                }
                for btn in mouse_up {
                    lock.mouse_up(btn);
                }
                if scroll_delta != 0.0 {
                    lock.mouse_scroll(scroll_delta);
                }

                for k in keys_down.iter() {
                    lock.key_down(*k);
                }
                for k in keys_up.iter() {
                    lock.key_up(*k);
                }

                // update, then check if complete
                lock.update();
                if lock.completed {
                    println!("beatmap complete");
                    let score = &lock.score;
                    let replay = &lock.replay;

                    if !lock.replaying {
                        // save score
                        save_score(&score);
                        match save_replay(&replay, &score) {
                            Ok(_)=> println!("replay saved ok"),
                            Err(e) => println!("error saving replay: {}", e),
                        }
                        // match save_all_scores() {
                        //     Ok(_) => println!("Scores saved successfully"),
                        //     Err(e) => println!("Failed to save scores! {}", e),
                        // }
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

                    }

                    // used to indicate user stopped watching a replay
                    if lock.replaying && !lock.started {
                        // go back to beatmap select
                        let menu = self.menus.get("beatmap").unwrap();
                        let menu = menu.clone();
                        self.queue_state_change(GameState::InMenu(menu));
                    } else {
                        // show score menu
                        let menu = ScoreMenu::new(&score, lock.beatmap.metadata.clone());
                        self.queue_state_change(GameState::InMenu(Arc::new(Mutex::new(menu))));
                    }
                }
            }
            
            GameState::InMenu(ref menu) => {
                let mut menu = menu.lock();

                // menu input events

                // clicks
                for b in mouse_down { 
                    // game.start_map() can happen here, which needs &mut self
                    menu.on_click(mouse_pos, b, mods, self);
                }
                // mouse move
                if mouse_moved {menu.on_mouse_move(mouse_pos, self)}
                // mouse scroll
                if scroll_delta.abs() > 0.0 {menu.on_scroll(scroll_delta, self)}

                //check keys down
                for key in keys_down {menu.on_key_press(key, self, mods)}
                //TODO: check keys up (or remove it, probably not used anywhere)

                // check text
                if text.len() > 0 {menu.on_text(text);}

                // window focus change
                if let Some(has_focus) = window_focus_changed {
                    menu.on_focus_change(has_focus, self);
                }

                menu.update(self);
            }

            GameState::Spectating(ref data, ref state, ref _beatmap) => {
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

            GameState::None => {
                // might be transitioning
                if self.transition.is_some().clone() && elapsed - self.transition_timer > TRANSITION_TIME / 2 {
                    let trans = self.transition.as_ref().unwrap().clone();
                    self.queue_state_change(trans);
                    self.transition = None;
                    self.transition_timer = elapsed;
                }
            }

            _ => {}
        }
        
        // update game mode
        match &self.queued_state {
            // queued mode didnt change, set the unlocked's mode to the updated mode
            GameState::None => self.current_state = current_state,
            GameState::Closing => {
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

                match &self.queued_state {
                    GameState::Ingame(manager) => {
                        let (m, h) = {
                            let mut lock = manager.lock();
                            lock.start();

                            (lock.beatmap.metadata.clone(), lock.beatmap.hash.clone())
                        };

                        self.set_background_beatmap(&m);
                        // if let Ok(t) = opengl_graphics::Texture::from_path(m.image_filename.clone(), &opengl_graphics::TextureSettings::new()) {
                        //     self.background_image = Some(Image::new(Vector2::zero(), f64::MAX, t, window_size()));
                        // } else {
                        //     self.background_image = None;
                        // }

                        let text = format!("{}-{}[{}]\n{}", m.artist, m.title, m.version, h);
                        self.threading.spawn(async move {
                            OnlineManager::set_action(online_manager, UserAction::Ingame, text).await;
                        });
                    },
                    GameState::InMenu(_) => {
                        if let GameState::InMenu(menu) = &self.current_state {
                            if menu.lock().get_name() == "pause" {
                                if let Some(song) = Audio::get_song() {
                                    song.play();
                                }
                            }
                        }

                        self.threading.spawn(async move {
                            OnlineManager::set_action(online_manager, UserAction::Idle, String::new()).await;
                        });
                    },
                    GameState::Closing => {
                        // send logoff
                        self.threading.spawn(async move {
                            OnlineManager::set_action(online_manager, UserAction::Leaving, String::new()).await;
                        });
                    }
                    _ => {}
                }

                let mut do_transition = true;
                match &self.current_state {
                    GameState::None => do_transition = false,
                    GameState::InMenu(menu) if menu.lock().get_name() == "pause" => do_transition = false,
                    _ => {}
                }

                if do_transition {
                    // do a transition
                    let qm = &self.queued_state;
                    self.transition = Some(qm.clone());
                    self.transition_timer = elapsed;
                    self.transition_last = Some(self.current_state.clone());
                    self.queued_state = GameState::None;
                    self.current_state = GameState::None;
                } else {
                    // old mode was none, or was pause menu, transition to new mode
                    let mode = self.queued_state.clone();

                    self.current_state = mode.clone();
                    self.queued_state = GameState::None;

                    if let GameState::InMenu(menu) = &self.current_state {
                        menu.lock().on_change(true);
                    }
                }
            }
        }

        // update the notification manager
        NOTIFICATION_MANAGER.lock().update();
        
        // if timer.elapsed().as_secs_f32() * 1000.0 > 1.0 {
        //     println!("update took a while: {}", timer.elapsed().as_secs_f32() * 1000.0);
        // }
    }

    fn render(&mut self, args: RenderArgs) {
        // let timer = Instant::now();
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
            window_size(),
            None
        )));

        // mode
        match &self.current_state {
            GameState::Ingame(manager) => manager.lock().draw(args, &mut renderables),
            GameState::InMenu(menu) => renderables.extend(menu.lock().draw(args)),
            
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
                window_size(),
                None
            )));

            // draw old mode
            match (&self.current_state, &self.transition_last) {
                (GameState::None, Some(GameState::InMenu(menu))) => renderables.extend(menu.lock().draw(args)),
                _ => {}
            }
        }

        // users list
        // TODO: move this to a "dialog"
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
        self.render_queue.extend(self.volume_controller.draw(args));

        // add the things we just made to the render queue
        self.render_queue.extend(renderables);

        // draw fps's
        self.fps_display.draw(&mut self.render_queue);
        self.update_display.draw(&mut self.render_queue);
        self.input_update_display.draw(&mut self.render_queue);

        // draw notifications
        
        // update the notification manager
        NOTIFICATION_MANAGER.lock().draw(&mut self.render_queue);

        // draw cursor
        let mouse_pressed = self.input_manager.mouse_buttons.len() > 0 
            || self.input_manager.key_down(settings.standard_settings.left_key)
            || self.input_manager.key_down(settings.standard_settings.right_key);
        self.render_queue.push(Box::new(Circle::new(
            Color::new(0.8, 0.0, 0.6, 1.0),
            -f64::MAX,
            self.input_manager.mouse_pos,
            if mouse_pressed {10.0} else {5.0} * settings.cursor_scale
        )));


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


        // if timer.elapsed().as_secs_f32() * 1000.0 > 1.0 {
        //     println!("render took a while: {}", timer.elapsed().as_secs_f32() * 1000.0);
        // }
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
    
    pub fn queue_state_change(&mut self, state:GameState) {self.queued_state = state}

    /// shortcut for setting the game's background texture to a beatmap's image
    pub fn set_background_beatmap(&mut self, beatmap:&BeatmapMeta) {
        // let mut helper = BenchmarkHelper::new("loaad image");

        let settings = opengl_graphics::TextureSettings::new();
        // helper.log("settings made", true);

        let buf: Vec<u8> = match std::fs::read(&beatmap.image_filename) {
            Ok(buf) => buf,
            Err(_) => {
                self.background_image = None;
                return;
            }
        };

        // let buf = file.unwrap();
        // helper.log("file read", true);

        let img = image::load_from_memory(&buf).unwrap();
        // helper.log("image created", true);
        let img = img.into_rgba8();
        // helper.log("format converted", true);
        
        let tex = opengl_graphics::Texture::from_image(&img, &settings);
        // helper.log("texture made", true);

        self.background_image = Some(Image::new(Vector2::zero(), f64::MAX, tex, window_size()));
        // helper.log("background set", true);

        // match opengl_graphics::Texture::from_path(beatmap.image_filename.clone(), &settings) {
        //     Ok(tex) => self.background_image = Some(Image::new(Vector2::zero(), f64::MAX, tex, window_size())),
        //     Err(e) => {
        //         println!("Error loading beatmap texture: {}", e);
        //         self.background_image = None; //TODO!: use a known good background image
        //     },
        // }
    }
}


#[derive(Clone)]
pub enum GameState {
    None, // use this as the inital game mode, but me sure to change it after
    Closing,
    Ingame(Arc<Mutex<IngameManager>>),
    InMenu(Arc<Mutex<dyn Menu<Game>>>),

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