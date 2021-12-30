use tokio::{sync::Mutex, net::TcpStream};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::protocol::Message};

use super::discord::Discord;
use super::online_user::OnlineUser;
use taiko_rs_common::packets::PacketId;
use taiko_rs_common::serialization::{SerializationReader, SimpleWriter};

use crate::prelude::*;

type WsWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

// url to connect to
const CONNECT_URL:&str = "ws://localhost:8080";

// how many frames do we buffer before sending?
// higher means less network use, but also could cause issues with slower events (ie paused)
// might need a workaround
const SPECTATOR_BUFFER_FLUSH_SIZE: usize = 20;

lazy_static::lazy_static! {
    pub static ref ONLINE_MANAGER:Arc<Mutex<OnlineManager>> = Arc::new(Mutex::new(OnlineManager::new()));
}

///
pub struct OnlineManager {
    pub connected: bool,
    pub users: HashMap<u32, Arc<Mutex<OnlineUser>>>, // user id is key
    pub discord: Discord,

    // pub chat: Chat,
    user_id: u32, // this user's id

    /// socket writer
    pub writer: Option<Arc<Mutex<WsWriter>>>,

    // buffers 
    /// buffer for incoming and outgoing spectator frames
    pub(crate) buffered_spectator_frames: SpectatorFrames,
    pub(crate) last_spectator_frame: Instant,
    
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
        }
    }
    pub async fn start(s: Arc<Mutex<Self>>) {
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
                    println!("pass: '{}'", password);

                    // send login packet
                    let data = SimpleWriter::new()
                        .write(PacketId::Client_UserLogin)
                        .write(settings.username.clone())
                        .write(password)
                        .done();
                    s.send_data(data).await;
                }

                while let Some(message) = reader.next().await {
                    match message {
                        Ok(Message::Binary(data)) => OnlineManager::handle_packet(s.clone(), data).await,
                        Ok(message) => println!("got something else: {:?}", message),

                        Err(oof) => {
                            println!("oof: {}", oof);
                            s.lock().await.connected = false;
                            s.lock().await.writer = None;
                            // reconnect?
                        }
                    }
                }
            },
            Err(oof) => {
                s.lock().await.connected = false;
                println!("could not accept connection: {}", oof);
            }
        }
    }

    async fn handle_packet(s: Arc<Mutex<Self>>, data:Vec<u8>) {
        let mut reader = SerializationReader::new(data);

        while reader.can_read() {
            let raw_id:u16 = reader.read();
            let packet_id = PacketId::from(raw_id);
            println!("[Client] got packet id {:?}", packet_id);

            match packet_id {
                // login
                PacketId::Server_LoginResponse => {
                    let user_id = reader.read_i32();
                    if user_id <= 0 {
                        println!("got bad login");
                    } else {
                        s.lock().await.user_id = user_id as u32;
                        println!("login success");

                        // [test] send spec request
                        if let Some(writer) = &s.lock().await.writer {
                            let _ = writer.lock().await.send(Message::Binary(SimpleWriter::new()
                                .write(PacketId::Client_Spectate)
                                .write(1)
                                .done()
                            )).await;
                        }
                    }
                }

                // user updates
                PacketId::Server_UserJoined => {
                    let user_id = reader.read_i32() as u32;
                    let username = reader.read_string();
                    println!("user id joined: {}", user_id);
                    s.lock().await.users.insert(user_id, Arc::new(Mutex::new(OnlineUser::new(user_id, username))));
                }
                PacketId::Server_UserLeft => {
                    let user_id = reader.read_u32();
                    println!("user id left: {}", user_id);
                    s.lock().await.users.remove(&user_id);
                }
                PacketId::Server_UserStatusUpdate => {
                    let user_id = reader.read_u32();
                    let action:UserAction = reader.read();
                    let action_text = reader.read_string();
                    let mode: crate::PlayMode = reader.read();
                    println!("got user status update: {}, {:?}, {} ({:?})", user_id, action, action_text, mode);
                    
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

                    println!("got message: `{}` from user id `{}` in channel `{}`", message, user_id, channel);
                }

                
                // spectator
                PacketId::Server_SpectatorFrames => {
                    let _sender_id = reader.read_u32();
                    let frames:SpectatorFrames = reader.read();
                    // println!("got {} spectator frames from the server", frames.len());
                    s.lock().await.buffered_spectator_frames.extend(frames);
                }
                PacketId::Server_SpectatorJoined => {
                    let speccing_user_id:u32 = reader.read();
                    if let Some(u) = s.lock().await.find_user_by_id(speccing_user_id) {
                        let username = &u.lock().await.username;
                        NotificationManager::add_text_notification(&format!("{} is now spectating", username), 2000.0, Color::GREEN);
                    } else {
                        NotificationManager::add_text_notification(&format!("A user is now spectating"), 2000.0, Color::GREEN);
                    }
                }

                PacketId::Unknown => {
                    println!("got unknown packet id {}, dropping remaining packets", raw_id);
                    continue;
                }

                p => {
                    println!("Got unhandled packet: {:?}", p);
                    continue;
                }
            }
        }
    }

    pub async fn set_action(s:Arc<Mutex<Self>>, action:UserAction, action_text:String, mode: PlayMode) {
        let mut s = s.lock().await;

        if let Some(writer) = &s.writer {
            println!("writing update");
            let p = SimpleWriter::new()
                .write(PacketId::Client_StatusUpdate)
                .write(action)
                .write(action_text.clone())
                .write(mode)
                .done();
            writer.lock().await.send(Message::Binary(p)).await.expect("error: ");

            if action == UserAction::Leaving {
                let p = SimpleWriter::new().write(PacketId::Client_LogOut).done();
                let _ = writer.lock().await.send(Message::Binary(p)).await;
            }
            
            s.discord.change_status(action_text.clone());

        } else {
            println!("oof, no writer");
        }
    }


    pub async fn send_data(&mut self, data:Vec<u8>) {
        if let Some(writer) = &self.writer {
            writer.lock().await.send(Message::Binary(data)).await.expect("error sending packet. oof");
        }
    }

    pub async fn send_spec_frames(s:Arc<Mutex<Self>>, frames:SpectatorFrames) {

        let mut lock = s.lock().await;
        lock.buffered_spectator_frames.extend(frames);

        let times_up = lock.last_spectator_frame.elapsed().as_secs_f32() > 1.0;

        if times_up || lock.buffered_spectator_frames.len() >= SPECTATOR_BUFFER_FLUSH_SIZE {
            let frames = std::mem::take(&mut lock.buffered_spectator_frames);
            if times_up {println!("sending spec buffer (time)")} else {println!("sending spec buffer (len)")}

            let data = SimpleWriter::new().write(PacketId::Client_SpectatorFrames).write(frames).done();
            lock.send_data(data).await;
            lock.last_spectator_frame = Instant::now();
        }

    }

    pub fn get_pending_spec_frames(&mut self) -> SpectatorFrames {
        std::mem::take(&mut self.buffered_spectator_frames)
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