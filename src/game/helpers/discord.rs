// this is nuked until i can find a proper discord rpc thing, or make my own .-.

// rustcord = "0.2.4"
// use rustcord::{Rustcord, EventHandlers, User, RichPresenceBuilder};
// use std::{ffi::NulError};

use crate::game::GameMode;

// const APP_ID:&str = "857981337423577109";

pub struct Discord {
    // client: Rustcord
}

impl Discord {
    pub fn new() -> Result<Discord, DiscordError> {
        Err(DiscordError::Whatever)
        // let client = Rustcord::init::<Self>(APP_ID, true, None)?;
        // Ok(Discord{client})
    }

    pub fn change_status(&mut self, _mode:GameMode) {
        // let presence;

        // match mode {
        //     GameMode::None => { // idle
        //         presence = RichPresenceBuilder::new()
        //             .state("Taiko.rs")
        //             .details("Idle")
        //             .large_image_key("icon")
        //             .large_image_text("Taiko.rs")
        //             // .small_image_key("amethyst")
        //             // .small_image_text("Amethyst")
        //             .build();
        //     },
        //     GameMode::Closing => {return}, // dispose of anything?
        //     GameMode::Ingame(beatmap) => { // playing map
                
        //         presence = RichPresenceBuilder::new()
        //             .state("Taiko.rs")
        //             .details(&format!("Playing {}", beatmap.lock().unwrap().metadata.version_string()))
        //             .large_image_key("icon")
        //             .large_image_text("Taiko.rs")
        //             // .small_image_key("amethyst")
        //             // .small_image_text("Amethyst")
        //             .build();
        //     },
        //     GameMode::InMenu(_menu) => { // in a menu (idle?)
        //         presence = RichPresenceBuilder::new()
        //             .state("Taiko.rs")
        //             .details("In a menu")
        //             .large_image_key("icon")
        //             .large_image_text("Taiko.rs")
        //             // .small_image_key("amethyst")
        //             // .small_image_text("Amethyst")
        //             .build();
        //     },
        // }

        // if let Err(e) = self.client.update_presence(presence) {
        //     println!("Error updating discord presence: {}", e);
        // }


    }
}

pub enum DiscordError {
    Whatever
}

// impl EventHandlers for Discord {
//     fn ready(user: User) {
//         println!("[Discord] User {}#{} logged in...", user.username, user.discriminator);
//     }
//     fn errored(code: i32, message: &str) {
//         println!("[Discord] Error: {} (code {})", message, code);
//     }
//     fn disconnected(code: i32, message: &str) {
//         println!("[Discord] Disconnected: {} (code {})", message, code);
//     }
// }