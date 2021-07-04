use std::sync::{Arc, Mutex};



use futures_channel::mpsc::UnboundedSender;
use futures_util::{future, pin_mut, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};


use crate::game::{SerializationReader, SerializationWriter};

use super::online_user::OnlineUser;
use super::packets::*;




const CONNECT_URL:&str = "ws://localhost:8080";


///
pub struct OnlineManager {
    pub users: Vec<Arc<Mutex<OnlineUser>>>,
    client: Option<UnboundedSender<Message>>,

    other_thing: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>
}

impl OnlineManager {
    pub fn new() -> OnlineManager {
        OnlineManager {
            users: Vec::new(),
            client: None,
            other_thing: None,
        }
    }
    pub async fn start(s: Arc<Mutex<Self>>) {
        // initialize the connection

        let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
        // tokio::spawn(read_stdin(stdin_tx));
        {
            s.lock().unwrap().client = Some(stdin_tx);
        }

        let (ws_stream, _) = connect_async(CONNECT_URL.to_owned()).await.expect("Failed to connect");
        println!("WebSocket handshake has been successfully completed");
        // self.other_thing = Some(ws_stream);

        let (write, read) = ws_stream.split();
        let stdin_to_ws = stdin_rx.map(Ok).forward(write);

        let ws_to_stdout = {
            read.for_each(|message| async {
                match message {
                    Ok(message) => {
                        let data = message.into_data();
                        // tokio::io::stdout().write_all(&data).await.unwrap();
                        s.lock().unwrap().handle_incoming(data);
                    },
                    Err(e) => {
                        println!("Error reading message: {}",e);
                    },
                }
            })
        };
        
        pin_mut!(stdin_to_ws, ws_to_stdout);
        future::select(stdin_to_ws, ws_to_stdout).await;
    } 

    pub async fn start2(s: Arc<Mutex<Self>>) {
        let mut data = SerializationWriter::new();
        data.write(PacketId::UserJoined);
        data.write_u32(1);

        let m:Message = Message::Binary(data.data());
        println!("sending user joined packet");
        s.lock().unwrap().client.as_ref().unwrap().unbounded_send(m).expect("poopoo");
    }

    pub fn handle_incoming(&mut self, data:Vec<u8>) {
        let mut reader = SerializationReader::new(data);

        while reader.can_read() {
            let packet_id:PacketId = reader.read();
            let packet_length:u64 = reader.read();

            match packet_id {
                PacketId::UserJoined => {
                    let user_id = reader.read_u32();
                    println!("user id joined: {}", user_id);
                },
                PacketId::UserLeft => {
                    let user_id = reader.read_u32();
                    println!("user id left: {}", user_id);
                },

                PacketId::Unknown(read_id) => {
                    let mut data = Vec::new();
                    while data.len() < packet_length as usize {data.push(reader.read_u8())}

                    println!("got unknown packet id {}", read_id);
                },
            }
        }
    }
}

