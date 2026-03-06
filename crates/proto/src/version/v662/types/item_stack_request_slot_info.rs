use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackRequestSlotInfo<V: ProtoVersion> {
    pub container_net_id: V::ContainerEnumName,
    pub slot: i8,
    #[endianness(var)]
    pub raw_id: i32,
}
