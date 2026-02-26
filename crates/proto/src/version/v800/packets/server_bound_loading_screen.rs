use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 312)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerBoundLoadingScreenPacket {
    pub packet_type: ServerBoundLoadingScreenPacketType,
    #[endianness(le)]
    pub loading_screen_id: Option<i32>
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum ServerBoundLoadingScreenPacketType {
    Unknown = 0,
    StartLoadingScreen = 1,
    EndLoadingScreen = 2,
}