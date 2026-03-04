use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackResponseContainerInfo<V: ProtoVersion> {
    pub container_net_id: i8,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub slots: Vec<V::ItemStackResponseSlotInfo>,
}
