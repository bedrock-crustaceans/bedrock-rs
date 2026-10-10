use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 327, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundControlSchemeSetPacket<V: ProtoVersion> {
    pub scheme: V::ControlScheme,
}
