pub mod types;
pub mod packets;
pub mod reflection;
pub mod serialization;
#[cfg(feature="test")] mod tests;
#[cfg(feature="server")] pub mod tables;

pub use types::*;
pub use downcast_rs::*;
pub use crate::reflection as reflect;
#[cfg(feature="server")] pub use sea_orm;

pub mod macros {
    pub use tataku_common_proc_macros::Reflect;
    pub use tataku_common_proc_macros::FromStr;
    pub use tataku_common_proc_macros::Serializable;
    pub(crate) use serde::{ Serialize, Deserialize };
    pub use tataku_common_proc_macros::PacketSerialization;
}
