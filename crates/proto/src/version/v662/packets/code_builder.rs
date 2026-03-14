use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 150)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CodeBuilderPacket {
    pub url: String,
    pub should_open_code_builder: bool,
}
