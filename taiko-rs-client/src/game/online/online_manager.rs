use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::net::TcpStream;
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::{WebSocketStream, connect_async, tungstenite::protocol::Message};

use crate::game::Settings;
use super::online_user::OnlineUser;
use taiko_rs_common::types::UserAction;
use taiko_rs_common::packets::PacketId;
use taiko_rs_common::serialization::{SerializationReader, SimpleWriter};

type WsWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

const CONNECT_URL:&str = "ws://localhost:8080";


///
pub struct OnlineManager {
    pub users: HashMap<u32, Option<Arc<Mutex<OnlineUser>>>>, // user id is key
    pub writer: Option<Arc<Mutex<WsWriter>>>,
    
    user_id: u32, // this user's id
}

impl OnlineManager {
    pub fn new() -> OnlineManager {
        OnlineManager {
            user_id: 0,
            users: HashMap::new(),
            writer: None,
        }
    }
    pub async fn start(s: Arc<Mutex<Self>>) {

        // initialize the connection
        match connect_async(CONNECT_URL.to_owned()).await {
            Ok((ws_stream, _)) => {

                let (writer, mut reader) = ws_stream.split();
                let writer = Arc::new(Mutex::new(writer));

                // send login packet
                let settings = Settings::get().clone();
                let p = SimpleWriter::new().write(PacketId::Client_UserLogin).write(settings.username.clone()).write(settings.password.clone()).done();
                writer.lock().await.send(Message::Binary(p)).await.expect("poopoo");
                drop(settings);

                s.lock().await.writer = Some(writer);

                while let Some(message) = reader.next().await {
                    match message {
                        Ok(Message::Binary(data)) => OnlineManager::handle_packet(s.clone(), data).await,
    
                        Ok(message) => {
                            println!("got something else: {:?}", message);
                        },
    
                        Err(oof) => {
                            println!("oof: {}", oof);
                            // reconnect?
                        },
                    }
                }
            },
            Err(oof) => {
                println!("could not accept connection: {}", oof);
            },
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


                PacketId::Server_UserJoined => {
                    let user_id = reader.read_u32();
                    println!("user id joined: {}", user_id);
                    s.lock().await.users.insert(user_id, None);
                },
                PacketId::Server_UserLeft => {
                    let user_id = reader.read_u32();
                    println!("user id left: {}", user_id);
                    s.lock().await.users.remove(&user_id);
                },




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
            if action == UserAction::Leaving {
                let p = SimpleWriter::new().write(PacketId::Client_LogOut).done();
                let _ = writer.lock().await.send(Message::Binary(p)).await;
            }
            let p = SimpleWriter::new().write(PacketId::Client_StatusUpdate).write(action).write(action_text).done();
            writer.lock().await.send(Message::Binary(p)).await.expect("error: ");
        }
    }
}

