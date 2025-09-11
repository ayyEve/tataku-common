use crate::macros::*;
use crate::reflection::*;
use crate::serialization::*;
use std::collections::HashSet;
use crate::types::network::multiplayer::LobbyUserState;

#[derive(Serialize, Deserialize)]
#[derive(Reflect, Serializable)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LobbyUser {
    pub user_id: u32,

    pub state: LobbyUserState,
    pub mods: HashSet<String>,
    pub speed: u16
}
