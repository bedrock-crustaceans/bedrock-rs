use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 147)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackRequestPacket<V: ProtoVersion> {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub requests: Vec<RequestsEntry<V>>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct RequestsEntry<V: ProtoVersion> {
    #[endianness(var)]
    pub client_request_id: u32,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub actions: Vec<V::ItemStackRequestActionType>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub strings_to_filter: Vec<String>,
    pub strings_to_filter_origin: V::TextProcessingEventOrigin,
}
