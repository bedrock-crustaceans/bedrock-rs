use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 25)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LevelEventPacket {
    #[endianness(var)]
    pub event_id: i32,
    #[endianness(le)]
    pub position: (f32, f32, f32),
    #[endianness(var)]
    pub data: i32,
}
