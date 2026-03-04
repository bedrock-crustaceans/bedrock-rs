use bedrockrs_macros::{gamepacket, ProtoCodec};
use vek::Vec3;
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 157)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MotionPredictionHintsPacket<V: ProtoVersion> {
    pub runtime_id: V::ActorRuntimeID,
    #[endianness(le)]
    pub motion: Vec3<f32>,
    pub on_ground: bool,
}