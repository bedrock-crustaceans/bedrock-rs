use super::super::types::ActorRuntimeID;
use bedrockrs_macros::{gamepacket, ProtoCodec};
use vek::Vec3;

#[gamepacket(id = 157)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MotionPredictionHintsPacket {
    pub runtime_id: ActorRuntimeID,
    #[endianness(le)]
    pub motion: Vec3<f32>,
    pub on_ground: bool,
}