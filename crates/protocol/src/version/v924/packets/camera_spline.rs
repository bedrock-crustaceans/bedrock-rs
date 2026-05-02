use crate::version::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 338)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraSplinePacket<V: ProtoVersion> {
    pub splines: Vec<CameraSplineDefinition<V>>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraSplineDefinition<V: ProtoVersion> {
    pub name: String,
    pub instruction: V::CameraSplineInstruction,
}
