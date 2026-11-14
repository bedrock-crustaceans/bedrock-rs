use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 334, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundDataDrivenUICloseAllScreensPacket {}
