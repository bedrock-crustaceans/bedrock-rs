use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 140, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SettingsCommandPacket {
    pub command: String,
    pub suppress_output: bool,
}
