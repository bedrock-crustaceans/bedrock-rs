use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 165)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SyncActorPropertyPacket {
    #[nbt]
    pub property_data: nbtx::Value, // TODO: NBT Structure
}
