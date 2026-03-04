use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 327)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundControlSchemeSetPacket<V: ProtoVersion> {
    pub scheme: V::ControlScheme,
}
