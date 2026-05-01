use crate::version::versions::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackResponseContainerInfo<V: ProtoVersion> {
    pub container_net_id: i8,

    pub slots: Vec<V::ItemStackResponseSlotInfo>,
}
