use bedrock_protocol_version::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackRequestSlotInfo<V: ProtoVersion> {
    pub container_name: V::FullContainerName,
    pub slot: i8,
    #[endianness(var)]
    pub raw_id: i32,
}
