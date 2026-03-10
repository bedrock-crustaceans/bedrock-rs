use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackResponseContainerInfo<V: ProtoVersion> {
    pub container_net_id: i8,

    pub slots: Vec<V::ItemStackResponseSlotInfo>,
}
