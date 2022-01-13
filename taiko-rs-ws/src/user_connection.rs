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
    pub addr: SocketAddr,
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

            writer: None,
            addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0))
        }
    }

    pub fn new(writer:Arc<Mutex<WsWriter>>, addr: SocketAddr) -> Self {
        Self {
            bot: false,
            user_id: 0,
            username: String::new(),
            action: UserAction::Idle,
            action_text: String::new(),
            mode: PlayMode::Standard,

            spectators: Vec::new(),
            
            writer: Some(writer),
            addr
        }
    }

    // remove spectator, returns true if user was removed, false if they were not spectating
    pub async fn remove_spectator(&mut self, other_user: &mut UserConnection) -> bool {

        // if the host is speccing this user, the user must stop spectating
        for (i, id) in self.spectators.iter().enumerate() {
            if id == &other_user.user_id {
                println!("[Spec] Removing {} from {}'s spectators", other_user.username, self.username);
                self.spectators.swap_remove(i);

                // packet to send 
                let p = create_packet!(Server_SpectatorLeft {user_id: other_user.user_id});

                // tell ourselves someone stopped spectating
                send_packet!(self.writer, p.clone());

                // tell the user they stopped spectating
                send_packet!(other_user.writer, p);

                return true
            }
        }
        

        false
    }
}

impl std::fmt::Display for UserConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{} (id {})}}", self.username, self.user_id)
    }
}