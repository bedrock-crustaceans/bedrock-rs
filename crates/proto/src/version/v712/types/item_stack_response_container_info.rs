use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackResponseContainerInfo<V: ProtoVersion> {
    pub container_name: V::FullContainerName,

    pub slots: Vec<V::ItemStackResponseSlotInfo>,
}
