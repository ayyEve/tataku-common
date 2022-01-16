use crate::{prelude::*, BOT_ACCOUNT};

pub fn exists<P: AsRef<std::path::Path>>(path: P) -> bool {
    path.as_ref().exists()
}

pub async fn setup_bot_account(peermap: &PeerMap) {
    // Create a new bot account
    let bot = Arc::new(Mutex::new(UserConnection::new_bot("Bot".to_owned())));

    // Add the bot account
    peermap
        .write()
        .await
        .insert(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0)), bot.lock().await.clone());

    let bot_id = bot.lock().await.user_id.clone();
    BOT_ACCOUNT.force_set((bot_id, bot));
}


/// helper to avoid get().unwrap() for OnceCells which we are 100% sure has data
pub trait OnceCellForce<T> {
    fn force_get(&self) -> &T;
    fn force_get_mut(&mut self) -> &mut T;
    
    fn force_set(&self, data: T);
}
impl<T> OnceCellForce<T> for OnceCell<T> {
    fn force_get(&self) -> &T {self.get().unwrap()}
    fn force_get_mut(&mut self) -> &mut T {self.get_mut().unwrap()}
    fn force_set(&self, data: T) {let _ = self.set(data);}
}

