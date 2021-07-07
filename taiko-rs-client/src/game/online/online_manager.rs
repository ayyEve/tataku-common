use std::collections::HashMap;
use std::sync::Arc;

use tokio::{sync::Mutex, net::TcpStream};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::protocol::Message};

use crate::game::Settings;
use super::online_user::OnlineUser;
use taiko_rs_common::types::UserAction;
use taiko_rs_common::packets::PacketId;
use taiko_rs_common::serialization::{SerializationReader, SimpleWriter};

type WsWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;


const CONNECT_URL:&str = "ws://localhost:8080";

///
pub struct OnlineManager {
    pub connected: bool,
    pub users: HashMap<u32, Arc<Mutex<OnlineUser>>>, // user id is key
    pub writer: Option<Arc<Mutex<WsWriter>>>,
    
    user_id: u32, // this user's id
}

impl OnlineManager {
    pub fn new() -> OnlineManager {
        OnlineManager {
            user_id: 0,
            users: HashMap::new(),
            writer: None,
            connected: false,
        }
    }
    pub async fn start(s: Arc<Mutex<Self>>) {
        // initialize the connection
        match connect_async(CONNECT_URL.to_owned()).await {
            Ok((ws_stream, _)) => {
                s.lock().await.connected = true;
                let (writer, mut reader) = ws_stream.split();
                let writer = Arc::new(Mutex::new(writer));

                // send login packet
                let settings = Settings::get().clone();
                let p = SimpleWriter::new().write(PacketId::Client_UserLogin).write(settings.username.clone()).write(settings.password.clone()).done();
                writer.lock().await.send(Message::Binary(p)).await.expect("poopoo");

                s.lock().await.writer = Some(writer);

                while let Some(message) = reader.next().await {
                    match message {
                        Ok(Message::Binary(data)) => OnlineManager::handle_packet(s.clone(), data).await,
                        Ok(message) => println!("got something else: {:?}", message),
    
                        Err(oof) => {
                            println!("oof: {}", oof);
                            s.lock().await.connected = false;
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
                    }
                },

                // user updates
                PacketId::Server_UserJoined => {
                    let user_id = reader.read_u32();
                    let username = reader.read_string();
                    println!("user id joined: {}", user_id);
                    s.lock().await.users.insert(user_id, Arc::new(Mutex::new(OnlineUser::new(user_id, username))));
                },
                PacketId::Server_UserLeft => {
                    let user_id = reader.read_u32();
                    println!("user id left: {}", user_id);
                    s.lock().await.users.remove(&user_id);
                },
                PacketId::Server_UserStatusUpdate => {
                    let user_id = reader.read_u32();
                    let action: UserAction = reader.read();
                    let action_text = reader.read_string();
                    println!("got user status update: {}, {:?}, {}", user_id, action, action_text);
                    
                    if let Some(e) = s.lock().await.users.get_mut(&user_id) {
                        let mut a = e.lock().await;
                        a.action = Some(action);
                        a.action_text = Some(action_text);
                    }
                }

                PacketId::Unknown => {
                    println!("got unknown packet id {}, dropping remaining packets", raw_id);
                    continue;
                },

                _ => {
                    println!("hjfd;lgsnjl;dkgsfl");
                    continue;
                }
            }
        }
    }

    pub async fn set_action(s:Arc<Mutex<Self>>, action:UserAction, action_text:String) {
        if let Some(writer) = &s.lock().await.writer {
            println!("writing update");
            let p = SimpleWriter::new().write(PacketId::Client_StatusUpdate).write(action).write(action_text).done();
            writer.lock().await.send(Message::Binary(p)).await.expect("error: ");

            if action == UserAction::Leaving {
                let p = SimpleWriter::new().write(PacketId::Client_LogOut).done();
                let _ = writer.lock().await.send(Message::Binary(p)).await;
            }
        } else {
            println!("oof, no writer");
        }
    }
}

