use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 305, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RefreshEntitlementsPacket {}
