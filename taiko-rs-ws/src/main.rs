use std::net::{Ipv4Addr, SocketAddrV4};
use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::Arc,
};

use tokio::sync::{Mutex, OnceCell};
use tokio::net::{TcpListener, TcpStream};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use sea_orm::{DbBackend, ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, Statement, FromQueryResult};
use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::protocol::Message};

use argon2::{
    Argon2,
    password_hash::{
        PasswordHash,
        PasswordVerifier
    }
};

type WsWriter = SplitSink<WebSocketStream<TcpStream>, Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, UserConnection>>>;

use taiko_rs_common::prelude::*;

pub static DATABASE:OnceCell<DatabaseConnection> = OnceCell::const_new();

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let addr = env::args().nth(1).unwrap_or_else(|| "0.0.0.0:8080".to_string());

    let state = PeerMap::new(Mutex::new(HashMap::new()));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    //#region bot account
    //Create a new bot account
    let bot = UserConnection::new_bot("Bot".to_owned());
    let bot = Arc::new(Mutex::new(bot));

    //Add the bot account
    state
        .lock()
        .await
        .insert(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0)), bot.lock().await.to_owned());
    //#endregion

    //#region database connection
    let db = sea_orm::Database::connect("postgres://postgres:Tacos98@192.168.0.50:5432/taikors?sslmode=disable")
        .await
        .expect("Error connecting to database");

    println!("db connected");
    DATABASE.set(db).unwrap();
    //#endregion

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(bot.clone(), state.clone(), stream, addr));
    }

    Ok(())
}

async fn handle_connection(bot_account: Arc<Mutex<UserConnection>>, peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr) {
    match accept_async(raw_stream).await {
        Ok(ws_stream) => {

            let (writer, mut reader) = ws_stream.split();
            let writer = Arc::new(Mutex::new(writer));

            let user_connection = UserConnection::new(writer.clone());
            peer_map.lock().await.insert(addr, user_connection.clone());
            
            while let Some(message) = reader.next().await {
                match message {
                    Ok(Message::Binary(data)) => handle_packet(data, &bot_account.lock().await.to_owned(), &peer_map, &addr).await,
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
                        println!("oof: {:?}", oof);

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
        Err(oof) => println!("could not accept connection: {:?}", oof),
    }
}

fn create_server_send_message_packet(id: u32, message: String, channel: String) -> Message {
    Message::Binary(SimpleWriter::new()
        .write(PacketId::Server_SendMessage)
        .write(id)
        .write(message)
        .write(channel).done())
}

async fn create_server_score_update_packet(user_id: u32, mode: PlayMode) -> Message {
    let user_data = get_user_score_info(user_id, mode).await;

    let user_ranked_score: i64 = user_data.0;
    let user_total_score: i64 = user_data.1;
    let user_accuracy: f64 = user_data.2;
    let play_count: i32 = user_data.3;
    let rank: i32 = user_data.4;

    Message::Binary(SimpleWriter::new()
        .write(PacketId::Server_ScoreUpdate)
        .write(user_id)
        .write(user_total_score)
        .write(user_ranked_score)
        .write(user_accuracy)
        .write(play_count)
        .write(rank)
        .done()
    )
}

fn create_server_status_update_packet (user: &UserConnection) -> Message {
    Message::Binary(SimpleWriter::new()
        .write(PacketId::Server_UserStatusUpdate)
        .write(user.user_id)
        .write(user.action)
        .write(user.action_text.clone())
        .write(user.mode)
        .done())
}

async fn get_user_score_info(user_id: u32, mode: PlayMode) -> (i64, i64, f64, i32, i32) {
    let mut ranked_score = 0 as i64;
    let mut total_score = 0 as i64;
    let mut accuracy = 0.0;
    let mut playcount = 0;

    let mut rank = 0;

    match user_data_table::Entity::find()
        .filter(user_data_table::Column::Mode.eq(mode as i16))
        .filter(user_data_table::Column::UserId.eq(user_id))
        .one(DATABASE.get().unwrap())
        .await {
        Ok(user_data) => {
            match user_data {
                Some(user_data) => {
                    ranked_score = user_data.ranked_score;
                    total_score = user_data.total_score;
                    accuracy = user_data.accuracy;
                    playcount = user_data.play_count;
                }
                None => { }
            };
        },
        Err(_e) => { }
    }

    #[derive(Debug, FromQueryResult)]
    struct RankThing {
        rank: i64
    }

    let things: Vec<RankThing> = RankThing::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"SELECT rank FROM (SELECT user_id, ROW_NUMBER() OVER(ORDER BY ranked_score DESC) AS rank FROM user_data WHERE mode=$1) t WHERE user_id=$2"#,
        vec![(mode as i32).into(), (user_id as i32).into()],
    ))
        .all(DATABASE.get().unwrap())
        .await
        .unwrap();

    match things.first() {
        Some(thing) => {
            rank = thing.rank;
        }
        None => { }
    };

    (ranked_score, total_score, accuracy, playcount, rank as i32)
}

async fn handle_packet(data: Vec<u8>, bot_account: &UserConnection, peer_map: &PeerMap, addr: &SocketAddr) {
    let mut user_connection = peer_map.lock().await.get(addr).unwrap().clone();
    let mut reader = SerializationReader::new(data);

    let writer = user_connection.writer.clone().expect("We are somehow a bot? this socket doesnt even exist");
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
                let password:String = reader.read();

                // verify username and password
                let user_id;

                let user: Option<users_table::Model> = users_table::Entity::find()
                    .filter(users_table::Column::Username.eq(username.clone()))
                    .one(DATABASE.get().unwrap())
                    .await
                    .unwrap();

                match user {
                    None => {
                        // Send the user not found response
                        writer.send(Message::Binary(SimpleWriter::new()
                            .write(PacketId::Server_LoginResponse)
                            .write(-1 as i32)
                            .done()
                        ))
                            .await
                            .expect("poop");
                        println!("user not found");

                        return;
                    }
                    Some(user) => {
                        user_id = user.user_id;

                        let argon2 = Argon2::default();

                        let parsed_hash = PasswordHash::new(&user.password).unwrap();
                        if let Err(e) = argon2.verify_password(password.as_ref(), &parsed_hash) {
                            // Send the password incorrect response
                            writer.send(Message::Binary(SimpleWriter::new()
                                .write(PacketId::Server_LoginResponse)
                                .write(-2 as i32)
                                .done()
                            )).await.expect("poop");
                            println!("password incorrect: {}", e);

                            return;
                        }
                    }
                }


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
                    if i_addr == addr {
                        // Tell the user about their own score
                        let p = create_server_score_update_packet(user.user_id, user.mode).await;
                        writer.send(p).await.expect("ono");

                        continue
                    }

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

                    // Tell the user that just joined about all the other users score values
                    let p = create_server_score_update_packet(user.user_id, user.mode).await;
                    writer.send(p).await.expect("ono");

                    // Update the statuses for all the users
                    let p = create_server_status_update_packet(user);
                    writer.send(p).await.expect("ono");
                }
            }

            // client statuses
            PacketId::Client_LogOut => {
                //userid
                let user_id = user_connection.user_id;
                
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

            //Sent by a client to notify the server to update their score for everyone
            PacketId::Client_NotifyScoreUpdate => {
                let user_id = user_connection.user_id;

                let p = create_server_score_update_packet(user_id, user_connection.mode).await;

                // Send all users the new score info
                for (i_addr, user) in peer_map.lock().await.iter() {
                    if i_addr == addr {
                        let _ = writer.send(p.clone()).await;

                        continue
                    }

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

                {
                    let mut u = peer_map.lock().await;
                    let mut u = u.get_mut(addr).unwrap();

                    u.action = action;
                    u.action_text = action_text;
                    u.mode = mode;

                    user_connection = u.clone();
                }
                
                // update everyone with the new user info
                let p = create_server_status_update_packet(&user_connection);

                for (i_addr, user) in peer_map.lock().await.iter() {
                    if i_addr == addr {
                        let _ = writer.send(p.clone()).await;

                        continue
                    }

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

                //Makes sure we dont send a message to ourselves, that would be silly
                if channel == user_connection.username {
                    let _ = writer.send(create_server_send_message_packet(
                        bot_account.user_id,
                        "You cant send a message to yourself silly!".to_owned(),
                        user_connection.username.clone()
                    )).await;

                    return
                }

                //Create the packet to send to all clients
                let p = create_server_send_message_packet(
                    userid, message.clone(), channel.clone()
                );

                let mut did_send = false;

                for (i_addr, user) in peer_map.lock().await.iter() {
                    //Send the message to ourselves without any more checks
                    if i_addr == addr {
                        let _ = writer.send(p.clone()).await;
                    
                        continue
                    }

                    //If this is a DM and the current iter user is not the recipient, then skip
                    if !channel.starts_with("#") {
                        if user.username != channel {
                            continue;
                        }
                    }

                    did_send = true;

                    //Send the message to all clients
                    match &user.writer {
                        Some(writer) => { 
                            let _ = writer.lock().await.send(p.clone()).await;
                        },
                        
                        None => ()
                    };
                }

                //Tell the user that the message was not delivered
                if !did_send {
                    let _ = writer.send(create_server_send_message_packet(
                        bot_account.user_id,
                        "That user/channel is not online or does not exist".to_owned(),
                        user_connection.username.clone()
                    )).await;
                } else {
                    //Add to the message history
                    if channel.starts_with("#") {
                        let message_history_entry: message_history_table::ActiveModel = message_history_table::ActiveModel {
                            userid: Set(userid as i64),
                            channel: Set(channel.clone()),
                            contents: Set(message.clone()),
                            ..Default::default()
                        };

                        let _ = message_history_entry.insert(DATABASE.get().unwrap()).await.unwrap();
                    }

                    println!("[{}] {}: {}", channel, user_connection.username, message);
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
    pub action: UserAction,
    pub action_text: String,
    pub mode: PlayMode,

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

            writer: Some(writer)
        }
    }
}