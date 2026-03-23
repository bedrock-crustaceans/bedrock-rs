use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 157)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MotionPredictionHintsPacket<V: ProtoVersion> {
    pub runtime_id: V::ActorRuntimeID,
    #[endianness(le)]
    pub motion: (f32, f32, f32),
    pub on_ground: bool,
}
