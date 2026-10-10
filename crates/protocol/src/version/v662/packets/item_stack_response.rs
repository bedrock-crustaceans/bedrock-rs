use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 148, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackResponsePacket<V: ProtoVersion> {
    pub responses: Vec<V::ItemStackResponseInfo>,
}
