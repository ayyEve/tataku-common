use crate::prelude::*;

const BANNER_DEPTH: f64 = -99000.0;
const BANNER_WPADDING:f64 = 5.0;


/// how long of a buffer should we have? (ms)
const SPECTATOR_BUFFER_OK_DURATION:f32 = 500.0;

pub struct SpectatorManager {
    pub frames: SpectatorFrames, 
    pub state: SpectatorState, 
    pub game_manager: Option<IngameManager>,

    /// what time we have data for
    /// ie, up to what time we can show gameplay
    pub good_until: f32,
    pub map_length: f32,


    // list of id,username for specs
    pub spectator_cache: HashMap<u32, String>
}
impl SpectatorManager {
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            state: SpectatorState::None,
            game_manager: None,
            good_until: 0.0,
            map_length: 0.0,
            spectator_cache: HashMap::new()
        }
    }

    pub fn update(&mut self, game: &mut Game) {
        // (try to) read pending data from the online manager
        match ONLINE_MANAGER.try_lock() {
            Ok(mut online_manager) => self.frames.extend(online_manager.get_pending_spec_frames()),
            Err(e) => println!("[SpectatorManager::update] failed to lock online manager, {}", e),
        }

        // check all incoming frames
        for (time, frame) in std::mem::take(&mut self.frames) {
            self.good_until = self.good_until.max(time as f32);

            println!("[Spec] packet: {:?}", frame);
            match frame {
                SpectatorFrameData::Play { beatmap_hash, mode, mods } => {
                    self.good_until = 0.0;
                    self.map_length = 0.0;
                    // user started playing a map
                    println!("[Spec] Host started playing map");

                    let mods:ModManager = serde_json::from_str(&mods).unwrap();
                    // find the map
                    let mut beatmap_manager = BEATMAP_MANAGER.lock();
                    match beatmap_manager.get_by_hash(&beatmap_hash) {
                        Some(map) => {
                            beatmap_manager.set_current_beatmap(game, &map, false, false);
                            match manager_from_playmode(mode, &map) {
                                Ok(manager) => {
                                    self.game_manager = Some(manager);

                                    // need a mutable reference
                                    let m = self.game_manager.as_mut().unwrap();
                                    m.apply_mods(mods);
                                    m.replaying = true;
                                    m.start();

                                    self.map_length = m.end_time;
                                    self.state = SpectatorState::Watching;
                                },
                                Err(e) => {
                                    NotificationManager::add_error_notification("Error loading spec beatmap", e);
                                },
                            }
                        }
                        None => {
                            // user doesnt have beatmap
                            NotificationManager::add_text_notification("You do not have the map!", 2000.0, Color::RED);
                        }
                    }
                }

                SpectatorFrameData::Pause => {
                    println!("[Spec] pause");
                    self.state = SpectatorState::Paused;
                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.pause();
                    }
                }
                SpectatorFrameData::UnPause => {
                    println!("[Spec] unpause");
                    self.state = SpectatorState::Watching;
                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.start();
                    }
                }
                SpectatorFrameData::Stop => {
                    println!("[Spec] stop");
                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.pause();
                        NotificationManager::add_text_notification("Host looking for map", 2000.0, Color::BLUE);
                    }
                }
                SpectatorFrameData::Buffer => {/*nothing to handle here*/},
                SpectatorFrameData::SpectatingOther { user_id } => {
                    NotificationManager::add_text_notification("Host speccing someone", 2000.0, Color::BLUE);
                }
                SpectatorFrameData::ReplayFrame { frame } => {
                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.replay.frames.push((time as f32, frame))
                    }
                }
                SpectatorFrameData::ScoreSync { score } => {
                    // received score update
                    println!("[Spec] got score update");
                    // we should buffer these, and check the time. 
                    // if the time is at the score time, we should update our score, 
                    // as this score is probably more accurate, or at the very least will update new spectators
                }

                SpectatorFrameData::ChangingMap => {
                    println!("[Spec] host changing maps");
                    self.state = SpectatorState::MapChanging;

                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.pause()
                    }
                }
            }
        }

        
        // check our current state
        match &self.state {
            SpectatorState::None => {
                // in this case, the user should really be allowed to browse menus etc in the mean time. we might have to meme this
            }
            SpectatorState::Buffering => {
                if let Some(manager) = self.game_manager.as_mut() {
                    // buffer twice as long as we need
                    let buffer_duration = (manager.time() + SPECTATOR_BUFFER_OK_DURATION * 2.0).clamp(0.0, self.map_length);

                    if self.good_until >= buffer_duration {
                        self.state = SpectatorState::Watching;
                        println!("[Spec] no longer buffering");
                        manager.start();
                    } else {
                        println!("[Spec] buffering");
                    }
                }
            }
            SpectatorState::Watching => {
                // currently watching someone
                if let Some(manager) = self.game_manager.as_mut() {
                    manager.update();
                    
                    let buffer_duration = (manager.time() + SPECTATOR_BUFFER_OK_DURATION * 2.0).clamp(0.0, self.map_length);
                    if self.good_until < buffer_duration {
                        self.state = SpectatorState::Buffering;
                        println!("[Spec] starting buffer");
                        manager.pause();
                    }
                }
            }
            SpectatorState::Paused => {},
            SpectatorState::MapChanging => {},
        }
    }

    pub fn draw(&mut self, args: RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if let Some(manager) = self.game_manager.as_mut() {
            manager.draw(args, list)
        }
        
        // draw spectator banner
        match &self.state {
            SpectatorState::None => {}
            SpectatorState::Watching => {}
            SpectatorState::Buffering => draw_banner("Buffering", list),
            SpectatorState::Paused => draw_banner("Host Paused", list),
            SpectatorState::MapChanging => draw_banner("Host Changing Map", list),
        }
    }



    pub fn mouse_scroll(&mut self, delta: f64, _game:&mut Game) {
        if let Some(manager) = self.game_manager.as_mut() {
            manager.mouse_scroll(delta)
        }
    }
    pub fn mouse_move(&mut self, pos:Vector2, _game:&mut Game) {
        if let Some(manager) = self.game_manager.as_mut() {
            manager.mouse_move(pos)
        }
    }
    pub fn mouse_down(&mut self, _pos:Vector2, button:MouseButton, _mods:KeyModifiers, _game:&mut Game) {
        if let Some(manager) = self.game_manager.as_mut() {
            manager.mouse_down(button);
        }
    }
    pub fn mouse_up(&mut self, _pos:Vector2, button:MouseButton, _mods:KeyModifiers, _game:&mut Game) {
        if let Some(manager) = self.game_manager.as_mut() {
            manager.mouse_up(button)
        }
    }

    pub fn key_down(&mut self, key:piston::Key, mods:KeyModifiers, game:&mut Game) {
        if key == piston::Key::Escape {
            let menu = game.menus.get("main").unwrap().clone();
            game.queue_state_change(GameState::InMenu(menu));
            // resume song if paused

            if let Some(song) = Audio::get_song() {
                if song.get_playback_state() == Ok(PlaybackState::Paused) {
                    let _ = song.play(false);
                }
            }
        }

        if let Some(manager) = self.game_manager.as_mut() {
            manager.key_down(key, mods)
        }
    }
    pub fn key_up(&mut self, key:piston::Key, _mods:KeyModifiers, _game:&mut Game) {
        if let Some(manager) = self.game_manager.as_mut() {
            manager.key_up(key)
        }
    }
}


fn draw_banner(text:&str, list: &mut Vec<Box<dyn Renderable>>) {
    let window_size = Settings::window_size();
    let font = get_font("main");

    let mut offset_text = Text::new(
        Color::BLACK,
        BANNER_DEPTH,
        Vector2::zero(), // centered anyways
        32,
        text.to_owned(),
        font.clone()
    );
    
    let text_width = offset_text.measure_text().x + BANNER_WPADDING;
    // center
    let rect = Rectangle::bounds_only(
        Vector2::new((window_size.x - text_width) / 2.0, window_size.y * 1.0/3.0), 
        Vector2::new( text_width + BANNER_WPADDING, 64.0)
    );
    offset_text.center_text(rect);
    // add
    list.push(visibility_bg(rect.pos, rect.size, BANNER_DEPTH + 10.0));
    list.push(Box::new(offset_text));
}
