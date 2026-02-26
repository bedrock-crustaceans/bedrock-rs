use bedrockrs_macros::{gamepacket, ProtoCodec};
use super::super::enums::AuthoritativeMovementMode;

#[gamepacket(id = 319)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetMovementAuthorityPacket {
    pub movement_mode: AuthoritativeMovementMode,
}