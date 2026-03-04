use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 167)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RemoveVolumeEntityPacket<V: ProtoVersion> {
    pub entity_network_id: V::EntityNetID,
    #[endianness(var)]
    pub dimension_type: i32,
}