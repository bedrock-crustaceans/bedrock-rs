use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 148)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackResponsePacket<V: ProtoVersion> {
    pub responses: Vec<V::ItemStackResponseInfo>,
}
