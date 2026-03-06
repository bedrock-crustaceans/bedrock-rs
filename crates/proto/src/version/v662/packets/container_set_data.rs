use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 51)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ContainerSetDataPacket<V: ProtoVersion> {
    pub container_id: V::ContainerID,
    #[endianness(var)]
    pub id: i32,
    #[endianness(var)]
    pub value: i32,
}
