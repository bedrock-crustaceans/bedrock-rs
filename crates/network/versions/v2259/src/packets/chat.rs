use bedrock_network_macros::{ProtoCodec, packet};

#[packet(id = 361)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ChatPacket {
    pub signed_chat: SignedChat,
    pub room_id: Option<String>,
    pub formatting: Option<ChatFormatting>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct SignedChat {
    pub key_id: String,
    pub sender_id: String,
    pub message_id: String,
    pub scan_text: String,
    pub redacted_text: String,
    pub author_name: String,
    pub signature: String,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ChatFormatting {
    pub name_prefix: Option<String>,
    pub name_postfix: Option<String>,
    pub message_prefix: Option<String>,
    pub message_postfix: Option<String>,
}
