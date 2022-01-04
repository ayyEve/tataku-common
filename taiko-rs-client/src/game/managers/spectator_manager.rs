use crate::prelude::*;

pub struct SpectatorManager {
    pub frames: SpectatorFrames, 
    pub state: SpectatorState, 
    pub game_manager: Option<IngameManager>,

    /// what time we have data for
    /// ie, up to what time we can show gameplay
    pub good_until: f32,
    pub map_length: f32,
}
impl SpectatorManager {
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            state: SpectatorState::None,
            game_manager: None,
            good_until: 0.0,
            map_length: 0.0
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
                SpectatorFrameData::Play { beatmap_hash, mode } => {
                    // user started playing a map
                    println!("[Spec] Host started playing map");

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
                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.pause();
                    }
                }
                SpectatorFrameData::UnPause => {
                    println!("[Spec] unpause");
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
                },
                SpectatorFrameData::ReplayFrame { frame } => {
                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.replay.frames.push((time as f32, frame))
                    }
                },
                SpectatorFrameData::ScoreSync { score } => {
                    // received score update
                    println!("got score update");
                    // we should buffer these, and check the time. 
                    // if the time is at the score time, we should update our score, 
                    // as this score is probably more accurate, or at the very least update new spectators
                },
            }
        }

        // check our current state
        

        /// how long of a buffer should we have? (ms)
        const SPECTATOR_BUFFER_OK_DURATION:f32 = 500.0;

        match &self.state {
            SpectatorState::None => {
                // in this case, the user should really be allowed to browse menus etc in the mean time. we might have to meme this
            },
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
            },
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
            },
            SpectatorState::Paused => {},
            SpectatorState::MapChanging => {},
        }
    }

    pub fn draw(&mut self, args: RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if let Some(manager) = self.game_manager.as_mut() {
            manager.draw(args, list)
        }
    }
}