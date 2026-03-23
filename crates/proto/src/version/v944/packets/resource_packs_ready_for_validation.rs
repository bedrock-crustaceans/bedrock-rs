use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 340)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ResourcePacksReadyForValidationPacket {}