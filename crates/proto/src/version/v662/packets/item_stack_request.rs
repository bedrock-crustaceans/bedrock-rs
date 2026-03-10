use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 147)]
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
