use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 192)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerStatsPacket {
    #[endianness(le)]
    pub server_time: f32,
    #[endianness(le)]
    pub network_time: f32,
}