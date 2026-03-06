use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 2)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayStatusPacket<V: ProtoVersion> {
    pub status: V::PlayStatus,
}
