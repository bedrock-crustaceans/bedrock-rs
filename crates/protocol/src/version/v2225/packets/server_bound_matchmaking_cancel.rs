use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 356)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerBoundMatchmakingCancelPacket {}
