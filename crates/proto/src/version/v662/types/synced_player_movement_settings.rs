use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct SyncedPlayerMovementSettings<V: ProtoVersion> {
    pub authority_mode: V::ServerAuthMovementMode,
    #[endianness(var)]
    pub rewind_history_size: i32,
    pub server_authoritative_block_breaking: bool,
}
