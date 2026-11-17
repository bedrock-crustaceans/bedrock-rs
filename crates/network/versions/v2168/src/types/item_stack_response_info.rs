use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackResponseInfo<V: ProtoVersion> {
    pub result: V::ItemStackNetResult,
    #[endianness(var)]
    pub client_request_id: i32,
    pub containers: Option<Option<Vec<V::ItemStackResponseContainerInfo>>>,
}
