use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 177)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ScriptMessagePacket {
    pub message_id: String,
    pub message_value: String,
}
