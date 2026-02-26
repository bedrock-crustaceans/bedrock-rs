use super::super::enums::ContainerID;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 51)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ContainerSetDataPacket {
    pub container_id: ContainerID,
    #[endianness(var)]
    pub id: i32,
    #[endianness(var)]
    pub value: i32,
}