#[cfg(feature="test")] mod tests;
pub mod types;
pub mod packets;
pub mod reflection;
pub mod serialization;

#[cfg(feature="server")] pub mod tables;

pub mod prelude {
    // only re-export tables for server-side stuff
    #[cfg(feature="server")] pub use sea_orm;
    #[cfg(feature="server")] pub use crate::tables::*;

    pub(crate) use std::collections::{ HashSet, HashMap };
    pub(crate) use serde::{ Serialize, Deserialize };

    pub use serde_json;
    pub use downcast_rs::*;

    pub use crate::types::*;
    pub use crate::packets::*;
    pub use crate::reflection::*;
    pub use crate::serialization::*;
    pub use tataku_common_proc_macros::Reflect;
    pub use tataku_common_proc_macros::Serializable;
    pub use tataku_common_proc_macros::PacketSerialization;
}