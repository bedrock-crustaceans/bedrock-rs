use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 147)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackRequestPacket<V: ProtoVersion> {
    pub requests: Vec<RequestsEntry<V>>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct RequestsEntry<V: ProtoVersion> {
    #[endianness(var)]
    pub client_request_id: u32,
    pub actions: Vec<V::ItemStackRequestActionType>,
    pub strings_to_filter: Vec<String>,
    pub strings_to_filter_origin: V::TextProcessingEventOrigin,
}
