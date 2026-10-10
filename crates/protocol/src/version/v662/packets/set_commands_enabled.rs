use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 59, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetCommandsEnabledPacket {
    pub commands_enabled: bool,
}
