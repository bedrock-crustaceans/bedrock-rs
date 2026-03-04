use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 56)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BlockActorDataPacket<V: ProtoVersion> {
    pub block_position: V::NetworkBlockPosition,
    #[nbt]
    pub actor_data_tags: nbtx::Value, // TODO: NBT Structure
}
