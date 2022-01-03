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

    // remove spectator, returns true if user was removed, false if they were not spectating
    pub async fn remove_spectator(&mut self, other_user: &mut UserConnection) -> bool {

        
        // if the host is speccing this user, the user must stop spectating
        for (i, id) in self.spectators.iter().enumerate() {
            if id == &other_user.user_id {
                self.spectators.swap_remove(i);

                // packet to send 
                let p = Message::Binary(SimpleWriter::new()
                    .write(PacketId::Server_SpectatorLeft)
                    .write(other_user.user_id)
                    .done()
                );

                // tell ourselves someone stopped spectating
                if let Some(writer) = self.writer.as_mut() {
                    let _ = writer.lock().await.send(p.clone()).await;
                }
                
                // tell the user they stopped spectating
                if let Some(writer) = other_user.writer.as_mut() {
                    let _ = writer.lock().await.send(p).await;
                }

                return true
            }
        }
        

        false
    }
}