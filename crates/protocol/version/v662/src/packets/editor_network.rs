use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 190, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct EditorNetworkPacket {
    #[nbt]
    pub binary_payload: nbtx::Value,
}
