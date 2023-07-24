use crate::prelude::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[derive(Serialize, Deserialize, Serializable)]
pub struct LobbyUser {
    pub user_id: u32,

    pub state: LobbyUserState,
    pub mods: HashSet<String>,
    pub speed: u16
}
