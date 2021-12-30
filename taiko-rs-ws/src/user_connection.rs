
use crate::prelude::*;


#[derive(Clone)]
pub struct UserConnection {
    pub bot: bool,
    pub user_id: u32,
    pub username: String,
    pub action: UserAction,
    pub action_text: String,
    pub mode: PlayMode,

    /// list of spectator ids
    pub spectators: Vec<u32>,

    pub writer: Option<Arc<Mutex<WsWriter>>>,
}
impl UserConnection {
    pub fn new_bot(bot: String) -> Self {
        Self {
            bot: true,
            user_id: u32::MAX,
            username: bot,
            action: UserAction::Idle,
            action_text: "Moderating the world!".to_owned(),
            mode: PlayMode::Standard,

            spectators: Vec::new(),
            

            writer: None
        }
    }

    pub fn new(writer:Arc<Mutex<WsWriter>>) -> Self {
        Self {
            bot: false,
            user_id: 0,
            username: String::new(),
            action: UserAction::Idle,
            action_text: String::new(),
            mode: PlayMode::Standard,

            spectators: Vec::new(),
            
            writer: Some(writer)
        }
    }
}