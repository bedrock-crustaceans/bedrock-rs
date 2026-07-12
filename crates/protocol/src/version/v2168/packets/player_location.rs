use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 326)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerLocationPacket {
    #[endianness(var)]
    pub target_entity_id: i64,
    pub update: PlayerLocationType,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum PlayerLocationType {
    Coordinates {
        #[endianness(var)]
        unknown: i32,
        #[endianness(le)]
        position: (f32, f32, f32),
    } = 0,
    Hide {
        #[endianness(var)]
        unknown: i32,
    } = 1,
}
