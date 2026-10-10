use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 51, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ContainerSetDataPacket<V: ProtoVersion> {
    pub container_id: V::ContainerID,
    #[endianness(var)]
    pub id: i32,
    #[endianness(var)]
    pub value: i32,
}
