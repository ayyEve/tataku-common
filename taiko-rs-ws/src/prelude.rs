pub use std::{
    env,
    sync::Arc,
    collections::HashMap,
    io::Error as IoError,
    net::{
        Ipv4Addr, 
        SocketAddr,
        SocketAddrV4
    },
};

pub use tokio::{
    net::{
        TcpStream,
        TcpListener,
    },
    sync::{
        Mutex,
        RwLock,
        OnceCell
    },
};
pub use futures_util::{SinkExt, StreamExt, stream::SplitSink};
pub use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::protocol::Message};

pub use serde::{
    Serialize, 
    Deserialize
};

pub use argon2::{
    Argon2,
    password_hash::{
        PasswordHash,
        PasswordVerifier
    }
};

pub use taiko_rs_common::prelude::*;
pub use PacketId::*;

// internal things
pub use crate::helpers::*;
pub use crate::database::*;
pub use crate::settings::*;
pub use crate::send_packet;
pub use crate::create_packet;
pub use crate::CHECK_DOUBLE_LOCK;
pub use crate::user_connection::*;

// types
pub type AMutex<T> = Arc<Mutex<T>>;
pub type ARwLock<T> = Arc<RwLock<T>>;

pub type ConnectionData = AMutex<UserConnection>;
pub type WsWriter = SplitSink<WebSocketStream<TcpStream>, Message>;
pub type PeerMap = ARwLock<HashMap<SocketAddr, UserConnection>>;