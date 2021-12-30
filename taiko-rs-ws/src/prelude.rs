pub use std::net::{Ipv4Addr, SocketAddrV4};
pub use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::Arc,
};

pub use tokio::sync::{Mutex, OnceCell};
pub use tokio::net::{TcpListener, TcpStream};
pub use futures_util::{SinkExt, StreamExt, stream::SplitSink};
pub use sea_orm::{DbBackend, ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, Statement, FromQueryResult};
pub use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::protocol::Message};

pub use argon2::{
    Argon2,
    password_hash::{
        PasswordHash,
        PasswordVerifier
    }
};

pub use crate::user_connection::*;
pub use taiko_rs_common::prelude::*;

pub type WsWriter = SplitSink<WebSocketStream<TcpStream>, Message>;
pub type PeerMap = Arc<Mutex<HashMap<SocketAddr, UserConnection>>>;