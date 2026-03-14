use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 119)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AvailableActorIdentifiersPacket {
    #[nbt]
    pub actor_info_list: nbtx::Value, // TODO: NBT Structure
}
