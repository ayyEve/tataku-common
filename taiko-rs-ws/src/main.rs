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
                    Ok(Message::Binary(data)) => handle_packet(data, &user_connection, &peer_map).await,

                    Ok(message) => {
                        println!("got something else: {:?}", message);
                    },

                    Err(oof) => {
                        println!("oof: {}", oof);
                        peer_map.lock().await.remove(&addr);
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

async fn handle_packet(data:Vec<u8>, user_connection:&UserConnection, peer_map: &PeerMap) {
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

                // send response
                writer.send(Message::Binary(SimpleWriter::new().write(PacketId::Server_LoginResponse).write(1 as i32).done())).await.expect("ppoop");
                
                // tell everyone we joined
                let p = Message::Binary(SimpleWriter::new().write(PacketId::Server_UserJoined).write(1 as i32).done());
                for (_, user) in peer_map.lock().await.iter() {
                    let _ = user.writer.lock().await.send(p.clone()).await;
                }
            },

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



// let mut reader = SerializationReader::new(bytes);
// while reader.can_read() {
//     let id:PacketId = reader.read();
//     let length:u64 = reader.read();
//     println!("got id {:?} with length {}", id, length);
    
//     match id {
//         PacketId::UserJoined => {
//             let user_id:u32 = reader.read();
//             println!("user joined: {}", user_id);
//         },
//         PacketId::UserLeft => {
//             let user_id:u32 = reader.read();
//             println!("user left: {}", user_id);
//         },

//         PacketId::Unknown(read_id) => {
//             let mut data:Vec<u8> = Vec::new();

//             for _ in 0..length {
//                 data.push(reader.read());
//             }

//             println!("got unknwon packet id: {}", read_id);
//         },
//     }
// }