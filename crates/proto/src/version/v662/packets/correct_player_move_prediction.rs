use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};
use vek::Vec3;

#[packet(id = 161)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CorrectPlayerMovePredictionPacket<V: ProtoVersion> {
    #[endianness(le)]
    pub position: Vec3<f32>,
    #[endianness(le)]
    pub velocity: Vec3<f32>,
    pub on_ground: bool,
    #[endianness(var)]
    pub tick: u64,
    pub prediction_type: V::PredictionType,
}
