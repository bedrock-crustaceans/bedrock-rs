use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 142, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CompletedUsingItemPacket<V: ProtoVersion> {
    #[endianness(le)]
    pub item_id: u16,
    pub item_use_method: V::ItemUseMethod,
}
