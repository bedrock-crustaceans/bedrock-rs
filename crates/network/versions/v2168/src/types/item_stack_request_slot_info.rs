use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackRequestSlotInfo<V: ProtoVersion> {
    pub container_name: V::FullContainerName,
    pub slot: i8,
    #[endianness(le)]
    pub raw_id: i32,
}
