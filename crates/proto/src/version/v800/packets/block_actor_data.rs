use super::super::types::NetworkBlockPosition;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 56)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BlockActorDataPacket {
    pub block_position: NetworkBlockPosition,
    #[nbt]
    pub actor_data_tags: nbtx::Value, // TODO: NBT Structure
}