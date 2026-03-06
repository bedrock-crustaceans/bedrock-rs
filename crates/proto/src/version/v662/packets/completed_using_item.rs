use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 142)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CompletedUsingItemPacket<V: ProtoVersion> {
    #[endianness(le)]
    pub item_id: u16,
    pub item_use_method: V::ItemUseMethod,
}
