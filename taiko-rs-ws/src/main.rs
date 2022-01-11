// mod server;
mod prelude;
mod helpers;
mod settings;
mod database;
mod user_connection;

use prelude::*;

// const LOG_USERS:bool = true;
pub const CHECK_DOUBLE_LOCK:bool = true;


pub static PEER_MAP:OnceCell<PeerMap> = OnceCell::const_new();
/// (bot_user_id, connection_data)
pub static BOT_ACCOUNT:OnceCell<(u32, ConnectionData)> = OnceCell::const_new();


#[macro_export]
macro_rules! send_packet {
    ($writer:expr, $data:expr) => {
        if let Some(writer) = &$writer {
            // this is probably not very reliable but its a good first-check
            if CHECK_DOUBLE_LOCK {
                if let Err(e) = writer.try_lock() {
                    println!("[Writer] double lock! ({}:{}): {}", file!(), line!(), e)
                }
            }

            match writer.lock().await.send(Message::Binary($data)).await {
                Ok(_) => {true},
                Err(e) => {
                    println!("[Writer] Error sending data ({}:{}): {}", file!(), line!(), e);
                    if let Err(e) = writer.lock().await.close().await {
                        println!("error closing connection: {}", e);
                    }
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
    let state = Arc::new(RwLock::new(HashMap::new()));

    // database connection
    Database::init(&settings).await;

    // Create the event loop and TCP listener we'll accept connections on.
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    // create bot account
    setup_bot_account(&state).await;

    // user logging to help debugging
    check_user_list(&state);

    // Let's spawn the handling of each connection in a separate task.
    println!("[Startup] Listening on {}", addr);
    while let Ok((stream, addr)) = listener.accept().await {
        // NOTE: addr's ip is always my reverse proxy host. i dont know if this could cause issues, 
        // but the port is different per connection somehow so imma assume its fine lol
        tokio::spawn(handle_connection(state.clone(), stream, addr));
    }

    println!("[Shutdown] server closing");
    Ok(())
}


async fn handle_connection(peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr) {
    match accept_async(raw_stream).await {
        Ok(ws_stream) => {
            let (writer, mut reader) = ws_stream.split();
            let writer = Arc::new(Mutex::new(writer));

            let user_connection = UserConnection::new(writer.clone(), addr);
            peer_map.write().await.insert(addr, user_connection.clone());

            while let Some(message) = reader.next().await {
                match message {
                    Ok(Message::Binary(data)) => handle_packet(data, &peer_map, &addr).await,
                    Ok(Message::Close(close_frame)) => {
                        let close_reason = match close_frame {
                            Some(close_frame) => close_frame.reason.to_string(),
                            None => "Close frame not found".to_owned()
                        };

                        // remove user from the map
                        peer_map.write().await.remove(&addr);

                        println!("[Connection] Connection closed: {}", close_reason);
                    }
                    Ok(message) => println!("[Connection] got something else: {:?}", message),

                    Err(oof) => {
                        println!("[Connection] oof: {:?}", oof);

                        let mut lock = peer_map.write().await;
                        let user = lock.get(&addr).unwrap();
                        
                        // tell everyone we left
                        let data = create_packet!(PacketId::Server_UserLeft, user.user_id);
                        for (i_addr, other) in lock.iter() {
                            if i_addr == &addr {continue}
                            send_packet!(other.writer, data.clone());
                        }

                        let _ = lock.remove(&addr);
                        let _ = writer.lock().await.close().await;
                        break;
                    }
                }
            }
        
        }
        Err(e) => println!("[Connection] Could not accept connection: {:?}", e),
    }
}


async fn handle_packet(data: Vec<u8>, peer_map: &PeerMap, addr: &SocketAddr) {
    let mut user_connection = peer_map.read().await.get(addr).unwrap().clone();
    let mut reader = SerializationReader::new(data);

    while reader.can_read() {
        let raw_id = reader.read();
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
                let user = Database::get_user_by_username(&username).await;

                // TODO: would be nice if we could shorten this as well
                let user_id = match user {
                    None => {
                        // Send the user not found response
                        println!("[Login] user not found: {}", username);
                        send_packet!(user_connection.writer, create_packet!(PacketId::Server_LoginResponse, -1 as i32));
                        return;
                    }
                    Some(user) => {
                        let argon2 = Argon2::default();
                        let parsed_hash = PasswordHash::new(&user.password).unwrap();
                        if let Err(e) = argon2.verify_password(password.as_ref(), &parsed_hash) {
                            // Send the password incorrect response
                            println!("[Login] password incorrect: {}", e);
                            send_packet!(user_connection.writer, create_packet!(PacketId::Server_LoginResponse, -2 as i32));
                            return;
                        }

                        user.user_id
                    }
                };


                {
                    let mut u = peer_map.write().await;
                    let mut u = u.get_mut(addr).unwrap();
                    u.user_id = user_id as u32;
                    u.username = username.clone();

                    // update the current one too
                    user_connection.user_id = user_id as u32;
                    user_connection.username = username.clone();
                }

                // Send the login response packet to the connecting user
                send_packet!(user_connection.writer, create_packet!(PacketId::Server_LoginResponse, user_id as i32));
                
                // tell everyone we joined
                let join_packet = create_packet!(PacketId::Server_UserJoined, user_id, user_connection.username.clone());
                
                for (_, user) in peer_map.read().await.iter() {
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

                for (i_addr, user) in peer_map.read().await.iter() {
                    if i_addr == addr {continue}
                    send_packet!(user.writer, data.clone());
                }
            }

            // =====  client statuses  =====

            //Sent by a client to notify the server to update their score for everyone
            PacketId::Client_NotifyScoreUpdate => {
                let data = create_server_score_update_packet(user_connection.user_id, user_connection.mode).await;

                // Send all users the new score info
                for (_, user) in peer_map.read().await.iter() {
                    send_packet!(user.writer, data.clone());
                }
            }

            // status update
            PacketId::Client_StatusUpdate => {
                let action: UserAction = reader.read();
                let action_text = reader.read_string();
                let mode: PlayMode = reader.read();

                {
                    let mut u = peer_map.write().await;
                    let mut u = u.get_mut(addr).unwrap();

                    u.action = action;
                    u.action_text = action_text;
                    u.mode = mode;

                    user_connection = u.clone();
                }
                
                // update everyone with the new user info
                let data = create_server_status_update_packet(&user_connection);
                for (_, user) in peer_map.read().await.iter() {
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
                        bot_id(),
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

                for (_, user) in peer_map.read().await.iter() {
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
                        bot_id(),
                        "That user/channel is not online or does not exist".to_owned(),
                        user_connection.username.clone()
                    );
                    
                    send_packet!(user_connection.writer, data);
                } else {
                    //Add to the message history
                    if channel.starts_with("#") {
                        Database::insert_into_message_history(userid, channel.clone(), message.clone()).await;
                    }

                    println!("[{}] {}: {}", channel, user_connection.username, message);
                }
            }



            // =====  spectator  =====
            PacketId::Client_Spectate => {
                // user wants to spectate
                let host_id = reader.read_u32();

                let mut found = false;
                for (conn, user) in peer_map.write().await.iter_mut() {
                    if conn == addr {continue}
                    
                    user.remove_spectator(&mut user_connection).await;
                    if user.user_id == host_id {
                        found = true;

                        if send_packet!(user.writer, create_packet!(PacketId::Server_SpectatorJoined, user_connection.user_id)) {
                            user.spectators.push(user_connection.user_id);
                        } else {
                            // trying to spectate a bot
                            let data = create_server_send_message_packet(
                                bot_id(),
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
                        bot_id(),
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
                for (conn, user) in peer_map.write().await.iter_mut() {
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

fn bot_id() -> u32 {
    BOT_ACCOUNT.force_get().0
}

fn check_user_list(map: &PeerMap) {
    // if !LOG_USERS {return}
    let cloned = map.clone();

    tokio::spawn(async move {
        loop {
            if let Err(e) = cloned.try_read() {
                println!("cant lock: {}", e);
            }

            for (i, u) in cloned.read().await.iter() {
                println!("i: {}, u: {}", i, u.username);
            }

            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
    });
}


// // tests
// #[cfg(test)]
// mod tests {
    
//     #[test]
//     fn load_test() {

//         // start the server in another thread
//         tokio::spawn(async move {
//             crate::main()
//         });


//         for i in 0..500 {

//         }
//     }
// }
