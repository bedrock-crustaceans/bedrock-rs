use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 190)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct EditorNetworkPacket {
    pub route_to_manager: bool,
    #[nbt]
    pub binary_payload: nbtx::Value, // TODO: NBT Structure
}
