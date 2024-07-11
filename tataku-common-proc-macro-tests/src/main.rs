use tataku_common::*;

#[test]
fn main() {
    #[derive(Debug)]
    #[derive(tataku_common_proc_macros::PacketSerialization)]
    #[Packet(type="u8")]
    enum Test {
        #[Packet(id=255)]
        Unknown,
    }
}