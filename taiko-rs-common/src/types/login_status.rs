use crate::prelude::*;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LoginStatus {
    /// some unknown error occurred
    UnknownError = 0,
    /// login success
    Ok = 1,
    /// password is incorrect
    BadPassword = 2,
    /// user doesnt exist
    NoUser = 3,
}
impl From<u8> for LoginStatus {
    fn from(n: u8) -> Self {
        match n {
            1 => Self::Ok,
            2 => Self::BadPassword,
            3 => Self::NoUser,

            _ => Self::UnknownError,
        }
    }
}
impl Serializable for LoginStatus {
    fn read(sr:&mut SerializationReader) -> Self {
        sr.read_u8().into()
    }

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write_u8(*self as u8)
    }
}