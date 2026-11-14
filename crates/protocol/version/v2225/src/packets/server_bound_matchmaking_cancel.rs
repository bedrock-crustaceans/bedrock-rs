use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 356, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerBoundMatchmakingCancelPacket {}
