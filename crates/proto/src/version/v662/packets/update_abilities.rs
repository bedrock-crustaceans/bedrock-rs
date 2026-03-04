use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 187)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateAbilitiesPacket<V: ProtoVersion> {
    pub data: V::SerializedAbilitiesData,
}