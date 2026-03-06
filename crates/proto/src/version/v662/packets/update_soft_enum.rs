use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 114)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateSoftEnumPacket<V: ProtoVersion> {
    pub enum_name: String,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub values: Vec<String>,
    pub update_type: V::SoftEnumUpdateType,
}
