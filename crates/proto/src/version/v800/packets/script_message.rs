use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 177)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ScriptMessagePacket {
    pub message_id: String,
    pub message_value: String,
}