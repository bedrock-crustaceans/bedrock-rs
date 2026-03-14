use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 140)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SettingsCommandPacket {
    pub command: String,
    pub suppress_output: bool,
}
