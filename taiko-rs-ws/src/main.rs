use std::net::{Ipv4Addr, SocketAddrV4};
use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::Arc,
};

use tokio::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::protocol::Message};

use taiko_rs_common::serialization::*;
use taiko_rs_common::packets::PacketId;
use taiko_rs_common::types::{PlayMode, UserAction};

type WsWriter = SplitSink<WebSocketStream<TcpStream>, Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, UserConnection>>>;


#[tokio::main]
async fn main() -> Result<(), IoError> {
    let addr = env::args().nth(1).unwrap_or_else(|| "0.0.0.0:8080".to_string());

    let state = PeerMap::new(Mutex::new(HashMap::new()));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    //Create a new bot account
    let bot = UserConnection::new_bot("Bot".to_owned());

    //Add the bot account
    state
        .lock()
        .await
        .insert(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0)), bot);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(state.clone(), stream, addr));
    }

    Ok(())
}

async fn handle_connection(peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr) {
    match accept_async(raw_stream).await {
        Ok(ws_stream) => {

            let (writer, mut reader) = ws_stream.split();
            let writer = Arc::new(Mutex::new(writer));

            let user_connection = UserConnection::new(writer.clone());
            peer_map.lock().await.insert(addr, user_connection.clone());
            
            while let Some(message) = reader.next().await {
                match message {
                    Ok(Message::Binary(data)) => handle_packet(data, &peer_map, &addr).await,
                    Ok(Message::Close(close_frame)) => {
                        let close_reason;

                        match close_frame {
                            Some(close_frame) => {
                                close_reason = close_frame.reason.to_string();
                            },
                            None => {
                                close_reason = "Close frame not found".to_owned();
                            }
                        }

                        println!("Connection closed: {}", close_reason);
                    }
                    Ok(message) => println!("got something else: {:?}", message),

                    Err(oof) => {
                        println!("oof: {}", oof);

                        let user = peer_map.lock().await.get(&addr).unwrap().clone();

                        // tell everyone we left
                        let p = Message::Binary(SimpleWriter::new().write(PacketId::Server_UserLeft).write(user.user_id).done());
                        
                        for (_, other) in peer_map.lock().await.iter() {
                            if user.user_id == other.user_id {continue}
                            match &other.writer {
                                Some(writer) => { 
                                    let _ = writer.lock().await.send(p.clone()).await; 
                                },
                                
                                None => ()
                            }
                        }

                        peer_map.lock().await.remove(&addr);
                        let _ = writer.lock().await.close();
                        break;
                    },
                }
            }
        },
        Err(oof) => println!("could not accept connection: {}", oof),
    }
}

async fn handle_packet(data:Vec<u8>, peer_map: &PeerMap, addr: &SocketAddr) {
    let user_connection = peer_map.lock().await.get(addr).unwrap().clone();
    let mut reader = SerializationReader::new(data);

    let writer = user_connection.writer.expect("We are somehow a bot? this socket doesnt even exist");
    let mut writer = writer.lock().await;

    while reader.can_read() {
        let raw_id:u16 = reader.read();
        let id = PacketId::from(raw_id);
        println!("[Server] got packet id {:?}", id);
        
        match id {
            PacketId::Client_UserLogin => {
                // read username
                // read password
                // get userid from database
                // return userid if good, 0 if bad
                let username:String = reader.read();
                let _password:String = reader.read();

                // verify username and password
                let user_id = peer_map.lock().await.len() as i32; //TODO
                {
                    let mut u = peer_map.lock().await;
                    let mut u = u.get_mut(addr).unwrap();
                    u.user_id = user_id as u32;
                    u.username = username.clone();
                }

                // Send the login response packet
                writer.send(
                    Message::Binary(SimpleWriter::new().write(PacketId::Server_LoginResponse).write(user_id as i32).done()
                )).await.expect("poop");
                
                // tell everyone we joined
                let p = Message::Binary(SimpleWriter::new().write(PacketId::Server_UserJoined).write(user_id as i32).write(username.clone()).done());
                
                for (i_addr, user) in peer_map.lock().await.iter() {
                    if i_addr == addr {continue}

                    //Send all the existing users about the new user
                    match &user.writer {
                        Some(writer) => { 
                            let _ = writer.lock().await.send(p.clone()).await; 
                        },
                        
                        None => ()
                    };

                    // Tell the user that just joined about all the other users
                    let p = Message::Binary(SimpleWriter::new().write(PacketId::Server_UserJoined).write(user.user_id).write(user.username.clone()).done());
                    writer.send(p).await.expect("ono");
                }
            }

            // client statuses
            PacketId::Client_LogOut => {
                //userid
                let user_id = user_connection.user_id;
                println!("user logging out: {}", user_id);
                
                
                // tell everyone we left
                let p = Message::Binary(SimpleWriter::new().write(PacketId::Server_UserLeft).write(user_id).done());
                for (i_addr, user) in peer_map.lock().await.iter() {
                    if i_addr == addr {continue}
                    
                    match &user.writer {
                        Some(writer) => { 
                            let _ = writer.lock().await.send(p.clone()).await; 
                        },
                        
                        None => ()
                    };
                }
            }   

            PacketId::Client_StatusUpdate => {
                let action: UserAction = reader.read();
                let action_text = reader.read_string();
                let mode: PlayMode = reader.read();

                println!("Got Status: {0} : {1}", u16::from(action), action_text);

                let user_id = user_connection.user_id;
                
                // update everyone with the new user info
                let p = Message::Binary(SimpleWriter::new()
                    .write(PacketId::Server_UserStatusUpdate)
                    .write(user_id)
                    .write(action)
                    .write(action_text)
                    .write(mode)
                    .done());
                for (i_addr, user) in peer_map.lock().await.iter() {
                    if i_addr == addr {continue}

                    match &user.writer {
                        Some(writer) => { 
                            let _ = writer.lock().await.send(p.clone()).await; 
                        },
                        
                        None => ()
                    };
                }
            }

            // chat messages
            PacketId::Client_SendMessage => {
                let userid = user_connection.user_id.clone();
                let message = reader.read_string();
                let channel = reader.read_string();

                println!("Got message {} from {} in channel {}", message, user_connection.username, channel);

                let p = Message::Binary(SimpleWriter::new()
                    .write(PacketId::Server_SendMessage)
                    .write(userid)
                    .write(message)
                    .write(channel.clone()).done());

                for (i_addr, user) in peer_map.lock().await.iter() {
                    if i_addr == addr {
                        let _ = writer.send(p.clone()).await;
                    
                        continue
                    }

                    if !channel.starts_with("#") {
                        if user.username != channel {
                            continue;
                        }
                    }

                    match &user.writer {
                        Some(writer) => { 
                            let _ = writer.lock().await.send(p.clone()).await; 
                        },
                        
                        None => ()
                    };
                }
            }

            // spectator?

            // multiplayer?

            PacketId::Unknown => {
                println!("got unknown packet id {}, dropping remaining packets", raw_id);
                continue;
            }

            n => {
                println!("got server packet {:?} somehow yikes", n);
                continue;
            }
        }
    }
}

#[derive(Clone)]
struct UserConnection {
    pub bot: bool,
    pub user_id: u32,
    pub username: String,

    pub writer: Option<Arc<Mutex<WsWriter>>>,
}
impl UserConnection {
    pub fn new_bot(bot: String) -> Self {
        Self {
            bot: true,
            user_id: u32::MAX,
            username: bot,

            writer: None
        }
    }

    pub fn new(writer:Arc<Mutex<WsWriter>>) -> Self {
        Self {
            bot: false,
            user_id: 0,
            username: String::new(),

            writer: Some(writer)
        }
    }
}