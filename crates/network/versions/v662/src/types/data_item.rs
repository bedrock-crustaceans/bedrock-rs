use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct DataItem<V: ProtoVersion> {
    pub data_item_id: V::ActorDataIDs,
    pub data_item_type: V::DataItemType,
}
