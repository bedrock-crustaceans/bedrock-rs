use crate::version::versions::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackResponseContainerInfo<V: ProtoVersion> {
    pub container_name: V::FullContainerName,

    pub slots: Vec<V::ItemStackResponseSlotInfo>,
}
