use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 20)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PassengerJumpPacket {
    #[endianness(var)]
    pub jump_scale: i32,
}
