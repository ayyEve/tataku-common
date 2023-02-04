pub use tataku_common::*;

#[test]
fn test() {
    #[derive(tataku_proc_macros::PacketSerialization)]
    #[Packet(type="u8")]
    enum Test {
        #[Packet(id=255)]
        Unknown,
    }
}