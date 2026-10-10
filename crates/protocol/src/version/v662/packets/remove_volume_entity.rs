use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 167, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RemoveVolumeEntityPacket<V: ProtoVersion> {
    pub entity_network_id: V::EntityNetID,
    #[endianness(var)]
    pub dimension_type: i32,
}
