use crate::prelude::*;

pub struct SpectatorManager {
    pub frames: SpectatorFrames, 
    pub state: SpectatorState, 
    pub game_manager: Option<IngameManager>,
}

impl SpectatorManager {
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            state: SpectatorState::None,
            game_manager: None
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
            match frame {
                SpectatorFrameData::Play { beatmap_hash, mode } => {
                    // user started playing a map
                    println!("Host started playing map");

                    let mut beatmap_manager = BEATMAP_MANAGER.lock();

                    // find the map
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

                                    self.state = SpectatorState::Watching;
                                },
                                Err(e) => {
                                    NotificationManager::add_error_notification("Error loading spec beatmap", e);
                                },
                            }
                        }
                        None => {
                            // user doesnt have beatmap
                            NotificationManager::add_text_notification("you do not have the map!", 2000.0, Color::RED);
                        }
                    }

                    
                }

                SpectatorFrameData::Pause => {
                    println!("spec pause");
                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.pause();
                    }
                }
                SpectatorFrameData::UnPause => {
                    println!("spec unpause");
                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.start();
                    }
                }
                SpectatorFrameData::Stop => {
                    println!("spec stop");
                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.pause();
                        NotificationManager::add_text_notification("Host looking for map", 2000.0, Color::BLUE);
                    }
                }
                SpectatorFrameData::Buffer => {
                    // idk what this is
                },
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
                    println!("got score update")
                },
            }
        }

        const SPECTATOR_BUFFER_OK_SIZE:usize = 100;
        match &self.state {
            SpectatorState::None => {}, //panic!("spectator state is none!"),
            SpectatorState::Buffering => {
                // if self.frames.len() >= SPECTATOR_BUFFER_OK_SIZE {
                    self.state = SpectatorState::Watching;
                //     println!("no longer buffering");
                // } else {
                //     // waiting for packets
                //     println!("buffering...");
                // }
            },
            SpectatorState::Watching => {
                // currently watching someone
                if let Some(manager) = self.game_manager.as_mut() {
                    manager.update();
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