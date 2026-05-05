use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 114)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateSoftEnumPacket<V: ProtoVersion> {
    pub enum_name: String,
    pub values: Vec<String>,
    pub update_type: V::SoftEnumUpdateType,
}
