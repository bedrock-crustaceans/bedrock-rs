use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 340, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ResourcePacksReadyForValidationPacket {}