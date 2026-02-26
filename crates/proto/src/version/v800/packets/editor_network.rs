use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 190)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct EditorNetworkPacket {
    pub route_to_manager: bool,
    #[nbt]
    pub binary_payload: nbtx::Value, // TODO: NBT Structure
}