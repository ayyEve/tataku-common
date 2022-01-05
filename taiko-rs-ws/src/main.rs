mod prelude; use prelude::*;
mod helpers;
mod settings; use settings::*;
pub(crate) mod user_connection;
mod packet_helpers; use packet_helpers::*;

pub static DATABASE:OnceCell<DatabaseConnection> = OnceCell::const_new();

const CHECK_DOUBLE_LOCK:bool = true;
macro_rules! send_packet {
    ($writer: expr, $data:expr) => {
        if let Some(writer) = &$writer {
            // this is probably not very reliable but its a good first-check
            if CHECK_DOUBLE_LOCK {
                if let Err(e) = writer.try_lock() {
                    println!("writer double lock! ({}:{}): {}", file!(), line!(), e)
                }
            }

            match writer.lock().await.send(Message::Binary($data)).await {
                Ok(_) => {true},
                Err(e) => {
                    println!("Error sending data ({}:{}): {}", file!(), line!(), e);
                    false
                },
            }
        } else {
            false
        }
    }
}

#[macro_export]
macro_rules! create_packet {
    ($($item:expr),+) => {
        SimpleWriter::new()
        $(.write($item))+
        .done()
    };
}


#[tokio::main]
async fn main() -> Result<(), IoError> {
    // read settings
    let settings = Settings::load();
    let addr = format!("0.0.0.0:{}", settings.port);

    let state = Arc::new(Mutex::new(HashMap::new()));

    // Create the event loop and TCP listener we'll accept connections on.
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    println!("[Startup] Listening on {}", addr);

    //#region bot account
    // Create a new bot account
    let bot = Arc::new(Mutex::new(UserConnection::new_bot("Bot".to_owned())));

    // Add the bot account
    state
        .lock()
        .await
        .insert(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0)), bot.lock().await.to_owned());
    //#endregion

    //#region database connection
    let db = sea_orm::Database::connect(settings.postgres.connection_string())
        .await
        .expect("Error connecting to database");

    println!("[Startup] db connected");
    DATABASE.set(db).unwrap();
    //#endregion

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        println!("addr: {}", addr);
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
                        let close_reason = match close_frame {
                            Some(close_frame) => close_frame.reason.to_string(),
                            None => "Close frame not found".to_owned()
                        };

                        println!("[Connection] Connection closed: {}", close_reason);
                    }
                    Ok(message) => println!("[Connection] got something else: {:?}", message),

                    Err(oof) => {
                        println!("[Connection] oof: {:?}", oof);

                        if let Some(user) = peer_map.lock().await.remove(&addr) {
                            // tell everyone we left
                            let data = create_packet!(PacketId::Server_UserLeft, user.user_id);
                            
                            for (_, other) in peer_map.lock().await.iter() {
                                send_packet!(other.writer, data.clone());
                            }
                        }

                        let _ = writer.lock().await.close().await;
                        break;
                    },
                }
            }
        }
        Err(e) => println!("[Connection] Could not accept connection: {:?}", e),
    }
}


async fn get_user_score_info(user_id: u32, mode: PlayMode) -> (i64, i64, f64, i32, i32) {
    let mut ranked_score = 0;
    let mut total_score = 0;
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
        Err(e) => println!("[Database] Error: {}", e)
    }

    #[derive(Debug, FromQueryResult)]
    struct RankThing {rank: i64}

    let things: Vec<RankThing> = RankThing::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"SELECT rank FROM (SELECT user_id, ROW_NUMBER() OVER(ORDER BY ranked_score DESC) AS rank FROM user_data WHERE mode=$1) t WHERE user_id=$2"#,
        vec![(mode as i32).into(), (user_id as i32).into()],
    ))
        .all(DATABASE.get().unwrap())
        .await
        .unwrap();

    if let Some(thing) = things.first() {
        rank = thing.rank
    }

    (ranked_score, total_score, accuracy, playcount, rank as i32)
}

async fn handle_packet(data: Vec<u8>, bot_account: &UserConnection, peer_map: &PeerMap, addr: &SocketAddr) {
    let mut user_connection = peer_map.lock().await.get(addr).unwrap().clone();
    let mut reader = SerializationReader::new(data);

    while reader.can_read() {
        let raw_id:u16 = reader.read();
        let id = PacketId::from(raw_id);
        println!("[Packet] got packet id {:?}", id);
        
        match id {
            // login
            PacketId::Client_UserLogin => {
                // get userid from database
                // return userid if good, 0 if bad
                let username:String = reader.read(); // read username
                let password:String = reader.read(); // read password

                // verify username and password
                let user: Option<users_table::Model> = users_table::Entity::find()
                    .filter(users_table::Column::Username.eq(username.clone()))
                    .one(DATABASE.get().unwrap())
                    .await
                    .unwrap();

                // TODO: would be nice if we could shorten this as well
                let user_id = match user {
                    None => {
                        // Send the user not found response
                        send_packet!(user_connection.writer, create_packet!(PacketId::Server_LoginResponse, -1 as i32));
                        println!("[Login] user not found");
                        return;
                    }
                    Some(user) => {
                        let argon2 = Argon2::default();
                        let parsed_hash = PasswordHash::new(&user.password).unwrap();
                        if let Err(e) = argon2.verify_password(password.as_ref(), &parsed_hash) {
                            // Send the password incorrect response
                            send_packet!(user_connection.writer, create_packet!(PacketId::Server_LoginResponse, -2 as i32));
                            println!("[Login] password incorrect: {}", e);
                            return;
                        }

                        user.user_id
                    }
                };


                {
                    let mut u = peer_map.lock().await;
                    let mut u = u.get_mut(addr).unwrap();
                    u.user_id = user_id as u32;
                    u.username = username.clone();
                }

                // Send the login response packet to the connecting user
                send_packet!(user_connection.writer, create_packet!(PacketId::Server_LoginResponse, user_id as i32));
                
                // tell everyone we joined
                let join_packet = create_packet!(PacketId::Server_UserJoined, user_id as i32);
                
                for (_, user) in peer_map.lock().await.iter() {
                    // not sure if this is still necessary with this code
                    // if i_addr == addr {
                    //     // Tell the user about their own score
                    //     send_packet!(user_connection.writer, create_server_score_update_packet(user.user_id, user.mode).await);
                    //     continue
                    // }

                    //Send all the existing users about the new user
                    send_packet!(user.writer, join_packet.clone());

                    // Tell the user that just joined about all the other users
                    send_packet!(user_connection.writer, create_packet!(PacketId::Server_UserJoined, user.user_id, user.username.clone()));

                    // Tell the user that just joined about all the other users score values
                    send_packet!(user_connection.writer, create_server_score_update_packet(user.user_id, user.mode).await);

                    // Update the statuses for all the users
                    send_packet!(user_connection.writer, create_server_status_update_packet(user));
                }
            }

            // logout
            PacketId::Client_LogOut => {
                // tell everyone we left
                let data = create_packet!(PacketId::Server_UserLeft, user_connection.user_id);

                for (i_addr, user) in peer_map.lock().await.iter() {
                    if i_addr == addr {continue}
                    send_packet!(user.writer, data.clone());
                }
            }

            // =====  client statuses  =====

            //Sent by a client to notify the server to update their score for everyone
            PacketId::Client_NotifyScoreUpdate => {
                let data = create_server_score_update_packet(user_connection.user_id, user_connection.mode).await;

                // Send all users the new score info
                for (_, user) in peer_map.lock().await.iter() {
                    send_packet!(user.writer, data.clone());
                }
            }

            // status update
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
                let data = create_server_status_update_packet(&user_connection);
                for (_, user) in peer_map.lock().await.iter() {
                    send_packet!(user.writer, data.clone());
                }
            }

            
            // =====  chat  =====

            // chat messages
            PacketId::Client_SendMessage => {
                let userid = user_connection.user_id.clone();
                let message = reader.read_string();
                let channel = reader.read_string();

                //Makes sure we dont send a message to ourselves, that would be silly
                if channel == user_connection.username {
                    let data = create_server_send_message_packet(
                        bot_account.user_id,
                        "You cant send a message to yourself silly!".to_owned(),
                        user_connection.username.clone()
                    );
                    
                    send_packet!(user_connection.writer, data);
                    return
                }

                //Create the packet to send to all clients
                let data = create_server_send_message_packet(
                    userid, message.clone(), channel.clone()
                );

                let mut did_send = false;

                for (_, user) in peer_map.lock().await.iter() {
                    //Send the message to ourselves without any more checks
                    //If this is a DM and the current iter user is not the recipient, then skip
                    if !channel.starts_with("#") {
                        if user.username != channel {
                            continue;
                        }
                    }

                    did_send = true;

                    //Send the message to all clients
                    send_packet!(user.writer, data.clone());
                }

                //Tell the user that the message was not delivered
                if !did_send {
                    let data = create_server_send_message_packet(
                        bot_account.user_id,
                        "That user/channel is not online or does not exist".to_owned(),
                        user_connection.username.clone()
                    );
                    
                    send_packet!(user_connection.writer, data);
                } else {
                    //Add to the message history
                    if channel.starts_with("#") {
                        let message_history_entry: message_history_table::ActiveModel = message_history_table::ActiveModel {
                            user_id: Set(userid as i64),
                            channel: Set(channel.clone()),
                            contents: Set(message.clone()),
                            ..Default::default()
                        };

                        let _ = message_history_entry.insert(DATABASE.get().unwrap()).await.unwrap();
                    }

                    println!("[{}] {}: {}", channel, user_connection.username, message);
                }
            }



            // =====  spectator  =====
            PacketId::Client_Spectate => {
                // user wants to spectate
                let host_id = reader.read_u32();

                let mut found = false;
                for (conn, user) in peer_map.lock().await.iter_mut() {
                    if conn == addr {continue}
                    
                    user.remove_spectator(&mut user_connection).await;
                    if user.user_id == host_id {
                        found = true;

                        if send_packet!(user.writer, create_packet!(PacketId::Server_SpectatorJoined, user_connection.user_id)) {
                            user.spectators.push(user_connection.user_id);
                        } else {
                            // trying to spectate a bot
                            let data = create_server_send_message_packet(
                                bot_account.user_id,
                                "You cant spectate a bot!".to_owned(),
                                user_connection.username.clone()
                            );
                            send_packet!(user_connection.writer, data);
                        }
                    }
                }
                if !found {
                    // user wasnt found
                    let data = create_server_send_message_packet(
                        bot_account.user_id,
                        "That user was not found".to_owned(),
                        user_connection.username.clone()
                    );
                    send_packet!(user_connection.writer, data);
                }
            }
            PacketId::Client_SpectatorFrames => {
                // let count = reader.read();
                let frames: Vec<SpectatorFrame> = reader.read();
                // println!("forwarding {} frames to the following users: {:?}", frames.len(), user_connection.spectators);

                let data = create_packet!(PacketId::Server_SpectatorFrames, user_connection.user_id, frames);
                for (conn, user) in peer_map.lock().await.iter_mut() {
                    if conn == addr {continue}
                    
                    if user_connection.spectators.contains(&user.user_id) {
                        send_packet!(user.writer, data.clone());
                    }
                }
            }

            
            // ping and pong
            PacketId::Ping => {
                send_packet!(user_connection.writer, create_packet!(PacketId::Pong));
            }
            PacketId::Pong => {
                println!("[Packet] got pong from client");
            }


            // multiplayer?


            // Other
            PacketId::Unknown => {
                println!("[Packet] got unknown packet id {}, dropping remaining packets", raw_id);
                break;
            }

            n => {
                println!("[Packet] unhandled packet {:?}, dropping remaining packets", n);
                break;
            }
        }
    }
}

