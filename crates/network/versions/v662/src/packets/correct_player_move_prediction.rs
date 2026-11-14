use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 161, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CorrectPlayerMovePredictionPacket<V: ProtoVersion> {
    #[endianness(le)]
    pub position: (f32, f32, f32),
    #[endianness(le)]
    pub velocity: (f32, f32, f32),
    pub on_ground: bool,
    #[endianness(var)]
    pub tick: u64,
    pub prediction_type: V::PredictionType,
}
