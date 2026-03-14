use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 327)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundControlSchemeSetPacket<V: ProtoVersion> {
    pub scheme: V::ControlScheme,
}
