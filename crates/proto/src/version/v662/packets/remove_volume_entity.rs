use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 167)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RemoveVolumeEntityPacket<V: ProtoVersion> {
    pub entity_network_id: V::EntityNetID,
    #[endianness(var)]
    pub dimension_type: i32,
}
