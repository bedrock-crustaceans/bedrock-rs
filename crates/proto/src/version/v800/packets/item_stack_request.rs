use super::item_stack_request_packet::RequestsEntry;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 147)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackRequestPacket {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub requests: Vec<RequestsEntry>,
}

pub mod item_stack_request_packet {
    use bedrockrs_macros::ProtoCodec;
    use super::super::super::enums::{ItemStackRequestActionType, TextProcessingEventOrigin};
    use super::super::super::types::ItemStackRequestSlotInfo;

    #[derive(ProtoCodec, Clone, Debug)]
    pub struct RequestsEntry {
        #[endianness(var)]
        pub client_request_id: u32,
        #[vec_repr(u32)]
        #[vec_endianness(var)]
        pub actions: Vec<ItemStackRequestActionType>,
        #[vec_repr(u32)]
        #[vec_endianness(var)]
        pub strings_to_filter: Vec<String>,
        pub strings_to_filter_origin: TextProcessingEventOrigin,
    }
}


