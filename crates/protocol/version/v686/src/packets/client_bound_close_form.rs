use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 310, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundCloseFormPacket {}
