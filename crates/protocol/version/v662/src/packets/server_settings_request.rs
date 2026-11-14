use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 102, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerSettingsRequestPacket {}
