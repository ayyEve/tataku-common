use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::Arc,
};

use futures_util::FutureExt;
use tokio::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::protocol::Message};


use taiko_rs_common::serialization::*;
use taiko_rs_common::packets::PacketId;
use taiko_rs_common::types::UserAction;

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

                    Ok(message) => {
                        println!("got something else: {:?}", message);
                    },

                    Err(oof) => {
                        println!("oof: {}", oof);
                        peer_map.lock().await.remove(&addr);
                        let _ = writer.lock().await.close();
                        break;
                    },
                }
            }
        },
        Err(oof) => {
            println!("could not accept connection: {}", oof);
        },
    }
}

async fn handle_packet(data:Vec<u8>, peer_map: &PeerMap, addr: &SocketAddr) {
    let user_connection = peer_map.lock().await.get(addr).unwrap().clone();
    let mut reader = SerializationReader::new(data);
    let mut writer = user_connection.writer.lock().await;

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
                let _username:String = reader.read();
                let _password:String = reader.read();

                // verify username and password
                let user_id = 1; //TODO
                peer_map.lock().await.get_mut(addr).unwrap().user_id = user_id as u32;

                // tell everyone we joined
                let p = Message::Binary(SimpleWriter::new().write(PacketId::Server_UserJoined).write(user_id as i32).done());
                
                for (i_addr, user) in peer_map.lock().await.iter() {
                    if i_addr == addr {continue}
                    let _ = user.writer.lock().await.send(p.clone()).await;
                }

                // send response
                writer.send(Message::Binary(SimpleWriter::new().write(PacketId::Server_LoginResponse).write(user_id as i32).done())).await.expect("ppoop");
                
            },

            PacketId::Client_LogOut => {
                let user_id = user_connection.user_id;
                println!("user logging out: {}", user_id);
                
                // tell everyone we left
                let p = Message::Binary(SimpleWriter::new().write(PacketId::Server_UserLeft).write(user_id as i32).done());
                
                for (i_addr, user) in peer_map.lock().await.iter() {
                    if i_addr == addr {continue}
                    let _ = user.writer.lock().await.send(p.clone()).await;
                }
            }
            
            PacketId::Client_StatusUpdate => {
                let action:UserAction = reader.read();
                let action_text = reader.read_string();
                let user_id = user_connection.user_id;
                
                // update everyone
                let p = Message::Binary(SimpleWriter::new()
                    .write(PacketId::Server_UserStatusUpdate)
                    .write(user_id)
                    .write(action)
                    .write(action_text)
                    .done());
                for (i_addr, user) in peer_map.lock().await.iter() {
                    if i_addr == addr {continue}
                    let _ = user.writer.lock().await.send(p.clone()).await;
                }
            }


            PacketId::Unknown => {
                println!("got unknown packet id {}, dropping remaining packets", raw_id);
                continue;
            },

            n => {
                println!("got server packet {:?} somehow yikes", n);
                continue;
            },
        }
    }
}

#[derive(Clone)]
struct UserConnection {
    pub user_id: u32,
    pub username: String,

    pub writer: Arc<Mutex<WsWriter>>,
}
impl UserConnection {
    pub fn new(writer:Arc<Mutex<WsWriter>>) -> Self {

        Self {
            user_id: 0,
            username: String::new(),

            writer
        }
    }
}




// use std::{io::prelude::*, net::{TcpListener, TcpStream}};

// fn main_tcp() {
//     let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
//     listener.set_nonblocking(true).expect("oof");
//     let mut list = Vec::new();

//     for stream in listener.incoming() {
//         match stream {
//             Ok(stream) => {
//                 list.push(stream);
//             }
//             Err(e) => {}
//         }

//         for i in list.as_mut_slice() {
//             i.write("tacos are yum".as_bytes()).expect("oof2");
//         }
//     }
// }