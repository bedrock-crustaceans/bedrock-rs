use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackResponseInfo<V: ProtoVersion> {
    pub result: V::ItemStackNetResult,
    #[endianness(var)]
    pub client_request_id: i32,
    pub containers: Option<Vec<V::ItemStackResponseContainerInfo>>,
}
