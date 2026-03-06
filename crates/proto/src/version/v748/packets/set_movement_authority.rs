use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 319)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetMovementAuthorityPacket<V: ProtoVersion> {
    pub movement_mode: V::AuthoritativeMovementMode,
}
