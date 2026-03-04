use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackRequestSlotInfo<V: ProtoVersion> {
    pub container_name: V::FullContainerName,
    pub slot: i8,
    #[endianness(var)]
    pub raw_id: i32,
}
