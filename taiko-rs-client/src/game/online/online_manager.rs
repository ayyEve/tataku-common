use tokio::{sync::Mutex, net::TcpStream};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::protocol::Message};

use super::discord::Discord;
use super::online_user::OnlineUser;
use taiko_rs_common::packets::PacketId;
use taiko_rs_common::serialization::{SerializationReader, SimpleWriter};

use crate::prelude::*;

type WsWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;


const EXTRA_ONLINE_LOGGING:bool = false;

// url to connect to
#[cfg(feature = "gitlab_build")]
const CONNECT_URL:&str = "wss://taikors.ayyeve.xyz";
#[cfg(not(feature = "gitlab_build"))]
const CONNECT_URL:&str = "ws://127.0.0.1:8080";

// how many frames do we buffer before sending?
// higher means less packet spam
const SPECTATOR_BUFFER_FLUSH_SIZE: usize = 20;
type ThreadSafeSelf = Arc<Mutex<OnlineManager>>;

macro_rules! create_packet {
    ($id:ident) => {
        SimpleWriter::new()
        .write(PacketId::$id)
        .done()
    };
    ($id:ident, $($item:expr),+) => {
        SimpleWriter::new()
        .write(PacketId::$id)
        $(.write($item))+
        .done()
    };
}

macro_rules! send_packet {
    ($writer:expr, $data:expr) => {
        if let Some(writer) = &$writer {
            match writer.lock().await.send(Message::Binary($data)).await {
                Ok(_) => true,
                Err(e) => {
                    println!("[Writer] Error sending data ({}:{}): {}", file!(), line!(), e);
                    if let Err(e) = writer.lock().await.close().await {
                        println!("[Writer] error closing connection: {}", e);
                    }
                    false
                }
            }
        } else {
            false
        }
    }
}



lazy_static::lazy_static! {
    ///TODO: somehow change this to a RwLock. it should prioritize reads, as reads will almost always be syncronous
    pub static ref ONLINE_MANAGER:ThreadSafeSelf = Arc::new(Mutex::new(OnlineManager::new()));
}

///
pub struct OnlineManager {
    pub connected: bool,
    pub users: HashMap<u32, Arc<Mutex<OnlineUser>>>, // user id is key
    pub discord: Discord,

    // pub chat: Chat,
    pub user_id: u32, // this user's id

    /// socket writer
    pub writer: Option<Arc<Mutex<WsWriter>>>,

    // spectator specific vars

    // buffers 
    // is this user spectating someone?
    pub spectating: bool,
    /// buffer for incoming and outgoing spectator frames
    pub(crate) buffered_spectator_frames: SpectatorFrames,
    pub(crate) last_spectator_frame: Instant,

    pub(crate) spectator_list: Vec<(u32, String)>,
    /// which users are waiting for a spectator info response?
    /// TODO: should probably move the list itself to the server
    pub(crate) spectate_info_pending: Vec<u32>
}
impl OnlineManager {
    pub fn new() -> OnlineManager {
        OnlineManager {
            user_id: 0,
            users: HashMap::new(),
            discord: Discord::new(),
            // chat: Chat::new(),
            writer: None,
            connected: false,
            buffered_spectator_frames: Vec::new(),
            last_spectator_frame: Instant::now(),
            spectating: false,

            spectator_list: Vec::new(),
            spectate_info_pending: Vec::new(),
        }
    }
    pub async fn start(s: ThreadSafeSelf) {
        // initialize the connection
        match connect_async(CONNECT_URL.to_owned()).await {
            Ok((ws_stream, _)) => {
                s.lock().await.connected = true;
                let (writer, mut reader) = ws_stream.split();
                let writer = Arc::new(Mutex::new(writer));

                {
                    let mut s = s.lock().await;
                    s.writer = Some(writer);
                    let settings = Settings::get();

                    use sha2::Digest;
                    let mut hasher = sha2::Sha512::new();
                    hasher.update(settings.password.as_bytes());
                    let password = hasher.finalize();
                    let password = format!("{:02x?}", &password[..])
                        .replace(", ", "")
                        .trim_start_matches("[")
                        .trim_end_matches("]")
                        .to_owned();

                    // send login packet
                    send_packet!(s.writer, create_packet!(Client_UserLogin, settings.username.clone(), password));
                }

                while let Some(message) = reader.next().await {
                    match message {
                        Ok(Message::Binary(data)) => OnlineManager::handle_packet(s.clone(), data).await,
                        Ok(message) => if EXTRA_ONLINE_LOGGING {println!("[Online] got something else: {:?}", message)},

                        Err(oof) => {
                            println!("[Online] oof: {}", oof);
                            s.lock().await.connected = false;
                            s.lock().await.writer = None;
                            // reconnect?
                            break;
                        }
                    }
                }
            }
            Err(oof) => {
                s.lock().await.connected = false;
                println!("[Online] could not accept connection: {}", oof);
            }
        }
    }

    async fn handle_packet(s: ThreadSafeSelf, data:Vec<u8>) {
        let mut reader = SerializationReader::new(data);

        while reader.can_read() {
            let raw_id:u16 = reader.read();
            let packet_id = PacketId::from(raw_id);
            if EXTRA_ONLINE_LOGGING {println!("[Online] got packet id {:?}", packet_id)};

            match packet_id {
                // login
                PacketId::Server_LoginResponse => {
                    let user_id = reader.read_i32();
                    match user_id {
                        -2 => println!("[Login] user not found"),
                        -1 => println!("[Login] auth failed"),
                         0 => println!("[Login] Unknown Error"),
                         _ => {
                            s.lock().await.user_id = user_id as u32;
                            println!("[Login] success");
                            NotificationManager::add_text_notification("Logged in!", 2000.0, Color::GREEN);

                            ping_handler()

                            // [test] send spec request
                            // Self::start_spectating(1).await;
                        }
                    }
                }

                // user updates
                PacketId::Server_UserJoined => {
                    let user_id = reader.read();
                    let username = reader.read_string();
                    if EXTRA_ONLINE_LOGGING {println!("[Online] user {} joined (id: {})", username, user_id)};
                    s.lock().await.users.insert(user_id, Arc::new(Mutex::new(OnlineUser::new(user_id, username))));
                }
                PacketId::Server_UserLeft => {
                    let user_id = reader.read_u32();
                    if EXTRA_ONLINE_LOGGING {println!("[Online] user id {} left", user_id)};

                    let mut lock = s.lock().await;
                    // remove from online users
                    lock.users.remove(&user_id);

                    // remove from our spec list
                    for (i, &(id, _)) in lock.spectator_list.iter().enumerate() {
                        if id == user_id {
                            lock.spectator_list.swap_remove(i);
                            break;
                        }
                    }
                }
                PacketId::Server_UserStatusUpdate => {
                    let user_id = reader.read_u32();
                    let action:UserAction = reader.read();
                    let action_text = reader.read_string();
                    let _mode: crate::PlayMode = reader.read();
                    // println!("[Online] got user status update: {}, {:?}, {} ({:?})", user_id, action, action_text, mode);
                    
                    if let Some(e) = s.lock().await.users.get_mut(&user_id) {
                        let mut a = e.lock().await;
                        a.action = Some(action);
                        a.action_text = Some(action_text);
                    }
                }

                // score 
                PacketId::Server_ScoreUpdate => {
                    let _user_id:i32 = reader.read();
                    let _total_score:i64 = reader.read();
                    let _ranked_score:i64 = reader.read();
                    let _acc:f64 = reader.read();
                    let _play_count:i32 = reader.read();
                    let _rank:i32 = reader.read();
                }

                // chat
                PacketId::Server_SendMessage => {
                    let user_id:i32 = reader.read();
                    let message:String = reader.read();
                    let channel:String = reader.read();
                    if EXTRA_ONLINE_LOGGING {println!("[Online] got message: `{}` from user id `{}` in channel `{}`", message, user_id, channel)};
                }

                
                // spectator
                PacketId::Server_SpectatorFrames => {
                    let _sender_id = reader.read_u32();
                    let frames:SpectatorFrames = reader.read();
                    // println!("[Online] got {} spectator frames from the server", frames.len());
                    let mut lock = s.lock().await;
                    lock.buffered_spectator_frames.extend(frames);
                }
                // spec join/leave
                PacketId::Server_SpectatorJoined => {
                    let user_id:u32 = reader.read();
                    let user = if let Some(u) = s.lock().await.find_user_by_id(user_id) {
                        u.lock().await.username.clone()
                    } else {
                        "A user".to_owned()
                    };
                    s.lock().await.spectator_list.push((user_id, user.clone()));
                    
                    NotificationManager::add_text_notification(&format!("{} is now spectating", user), 2000.0, Color::GREEN);
                }
                PacketId::Server_SpectatorLeft => {
                    let user_id:u32 = reader.read();
                    let user = if let Some(u) = s.lock().await.find_user_by_id(user_id) {
                        u.lock().await.username.clone()
                    } else {
                        "A user".to_owned()
                    };
                    s.lock().await.spectator_list.remove_item((user_id, user.clone()));
                    
                    NotificationManager::add_text_notification(&format!("{} stopped spectating", user), 2000.0, Color::GREEN);
                }

                // spec info request
                PacketId::Server_SpectatorPlayingRequest => {
                    let requesting_user_id:u32 = reader.read();
                    s.lock().await.spectate_info_pending.push(requesting_user_id);

                    println!("[Online] got playing request");
                }


                // ping and pong
                PacketId::Ping => {
                    send_packet!(s.lock().await.writer, create_packet!(Pong));
                }
                PacketId::Pong => {
                    // println!("[Online] got pong from server");
                }

                // other packets
                PacketId::Unknown => {
                    println!("[Online] got unknown packet id {}, dropping remaining packets", raw_id);
                    break;
                }

                p => {
                    println!("[Online] Got unhandled packet: {:?}, dropping remaining packets", p);
                    break;
                }
            }
        }
    }

    pub async fn set_action(s: ThreadSafeSelf, action:UserAction, action_text:String, mode: PlayMode) {
        let mut s = s.lock().await;

        send_packet!(s.writer, create_packet!(Client_StatusUpdate, action, action_text.clone(), mode));
        if action == UserAction::Leaving {
            send_packet!(s.writer, create_packet!(Client_LogOut));
        }
        s.discord.change_status(action_text.clone());
    }


    // do things which require a reference to game
    pub fn do_game_things(&mut self, game: &mut Game) { 
        if self.spectate_info_pending.len() > 0 {

            // only get info if the current mode is ingame
            match &mut game.current_state {
                GameState::Ingame(manager) => {
                    for user_id in self.spectate_info_pending.iter() {
                        println!("[Online] sending playing request");
                        let packet = SpectatorFrameData::PlayingResponse {
                            user_id: *user_id,
                            beatmap_hash: manager.beatmap.hash(),
                            mode: manager.gamemode.playmode(),
                            mods: serde_json::to_string(&(*manager.current_mods)).unwrap(),
                            current_time: manager.time()
                        };

                        let clone = self.writer.clone();
                        tokio::spawn(async move {
                            send_packet!(clone, create_packet!(Client_SpectatorFrames, vec![(0, packet)]));
                        });
                    }
                    
                    self.spectate_info_pending.clear();
                }


                GameState::InMenu(menu) => {
                    match &*menu.lock().get_name() {
                        // if in a pause menu, dont clear the list, the user could enter the game again
                        // so we want to wait until they decide if they want to play or quit
                        "pause" => {}
                        _ => self.spectate_info_pending.clear()
                    }
                }

                // clear list for any other mode
                GameState::Closing
                |GameState::None
                |GameState::Spectating(_) => {
                    self.spectate_info_pending.clear();
                }
            }

        }
    }

    pub fn find_user_by_id(&self, user_id: u32) -> Option<Arc<Mutex<OnlineUser>>> {
        for (&id, user) in self.users.iter() {
            if id == user_id {
                return Some(user.clone())
            }
        }

        None
    }
}
impl OnlineManager {
    pub async fn send_spec_frames(s: ThreadSafeSelf, frames:SpectatorFrames, force_send: bool) {
        let mut lock = s.lock().await;
        // if we arent speccing, exit
        // hopefully resolves a bug
        // if !lock.spectating {return}


        lock.buffered_spectator_frames.extend(frames);
        let times_up = lock.last_spectator_frame.elapsed().as_secs_f32() > 1.0;

        if force_send || times_up || lock.buffered_spectator_frames.len() >= SPECTATOR_BUFFER_FLUSH_SIZE {
            let frames = std::mem::take(&mut lock.buffered_spectator_frames);
            // if force_send {println!("[Online] sending spec buffer (force)")} else if times_up {println!("[Online] sending spec buffer (time)")} else {println!("[Online] sending spec buffer (len)")}

            
            send_packet!(lock.writer, create_packet!(Client_SpectatorFrames, frames));
            lock.last_spectator_frame = Instant::now();
        }

    }

    pub async fn start_spectating(host_id: u32) {
        let mut s = ONLINE_MANAGER.lock().await;
        s.buffered_spectator_frames.clear();
        s.spectating = true;
        println!("[Online] speccing {}", host_id);

        send_packet!(s.writer, create_packet!(Client_Spectate, host_id));
    }

    pub async fn stop_spectating() {
        let mut s = ONLINE_MANAGER.lock().await;
        s.buffered_spectator_frames.clear();
        if !s.spectating {return}
        s.spectating = false;
        println!("[Online] stop speccing");
        
        send_packet!(s.writer, create_packet!(Client_SpectatorLeft));
    }

    pub fn get_pending_spec_frames(&mut self) -> SpectatorFrames {
        std::mem::take(&mut self.buffered_spectator_frames)
    }

}



const LOG_PINGS:bool = false;
fn ping_handler() {
    tokio::spawn(async move {
        let ping = create_packet!(Ping);
        let duration = std::time::Duration::from_millis(1000);

        loop {
            tokio::time::sleep(duration).await;
            if LOG_PINGS {println!("[Ping] sending ping")};
            send_packet!(ONLINE_MANAGER.lock().await.writer, ping.clone());
        }
    });
}



// tests
mod tests {
    const CONNECTION_COUNT: usize = 50;
    use crate::prelude::*;
    
    #[test]
    fn test() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            load_test().await
        });
        loop {}
    }

    async fn load_test() {
        for i in 0..CONNECTION_COUNT {
            tokio::spawn(async move {
                let thing = super::OnlineManager::new();
                let thing = Arc::new(tokio::sync::Mutex::new(thing));

                super::OnlineManager::start(thing).await;
                println!("online thread {} stopped", i);
            });
        }
    }
}
