use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 192, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerStatsPacket {
    #[endianness(le)]
    pub server_time: f32,
    #[endianness(le)]
    pub network_time: f32,
}
