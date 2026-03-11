use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistCategory<V: ProtoVersion> {
    pub name: String,
    pub entity_priorities: Vec<V::CameraAimAssistPriority>,
    pub block_priorities: Vec<V::CameraAimAssistPriority>,
    pub block_tag_priorities: Vec<V::CameraAimAssistPriority>,
    pub entity_type_families_priorities: Vec<V::CameraAimAssistPriority>,
    #[endianness(le)]
    pub entity_default_priorities: Option<i32>,
    #[endianness(le)]
    pub block_default_priorities: Option<i32>,
}
