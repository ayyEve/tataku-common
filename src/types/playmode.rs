pub type PlayMode = String;

pub fn playmode_from_u8(p:u8) -> PlayMode {
    match p {
        0 => "osu".to_owned(),
        1 => "taiko".to_owned(),
        2 => "catch".to_owned(),
        3 => "mania".to_owned(),
        4 => "adofai".to_owned(),
        5 => "pTyping".to_owned(),

        _ => String::new(),
    }
}
pub fn playmode_to_u8(s:PlayMode) -> u8 {
    match &*s {
        "osu" => 0,
        "taiko" => 1,
        "catch" => 2,
        "mania" => 3,
        "adofai" => 4,
        "pTyping" => 5,

        _ => 255
    }
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// #[derive(PacketSerialization, Serialize, Deserialize)]
// #[Packet(type="u8", gen_to_from)]
// pub enum PlayMode {
//     #[Packet(id=0)]
//     Standard,
//     #[Packet(id=1)]
//     Taiko,
//     #[Packet(id=2)]
//     Catch,
//     #[Packet(id=3)]
//     Mania,
//     #[Packet(id=4)]
//     Adofai,
//     #[allow(non_camel_case_types)]
//     #[Packet(id=5)]
//     pTyping,
    
//     #[Packet(id=255)]
//     Unknown = 255,
// }
// impl Default for PlayMode {
//     fn default() -> Self {
//         PlayMode::Standard
//     }
// }