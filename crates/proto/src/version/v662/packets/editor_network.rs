use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 190)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct EditorNetworkPacket {
    #[nbt]
    pub binary_payload: nbtx::Value,
}
