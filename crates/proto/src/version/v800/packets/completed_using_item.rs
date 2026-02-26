use super::super::enums::ItemUseMethod;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 142)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CompletedUsingItemPacket {
    #[endianness(le)]
    pub item_id: u16,
    pub item_use_method: ItemUseMethod,
}