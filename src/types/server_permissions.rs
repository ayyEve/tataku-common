use crate::prelude::*;

#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ServerPermissions {
    /// default state, ie no info
    None = 0,
    /// if set, this is a bot account
    /// if not set, this is a normal account
    Bot = 2,
    /// is this user a donator?
    Donator = 4,

    /// is this user a chat moderator?
    Moderator = 8,

    /// is this only a chat client
    ChatOnly = 16,
}
impl Serializable for Vec<ServerPermissions> {
    fn read(sr:&mut SerializationReader) -> Self {
        let num:u16 = sr.read();
        let mut list = Vec::new();

        macro_rules! check {
            ($($e:ident),+) => {
                $(if (num & ServerPermissions::$e as u16) > 0 {
                    list.push(ServerPermissions::$e)
                })+
            }
        }
        check!(Bot, Donator, Moderator);

        list
    }

    fn write(&self, sw:&mut SerializationWriter) {
        let mut num = 0;
        for i in self {
            num |= *i as u16
        }
        sw.write(num)
    }
}