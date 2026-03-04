use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 114)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateSoftEnumPacket<V: ProtoVersion> {
    pub enum_name: String,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub values: Vec<String>,
    pub update_type: V::SoftEnumUpdateType,
}