use super::super::enums::ServerAuthMovementMode;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct SyncedPlayerMovementSettings {
    pub authority_mode: ServerAuthMovementMode,
    #[endianness(var)]
    pub rewind_history_size: i32,
    pub server_authoritative_block_breaking: bool,
}