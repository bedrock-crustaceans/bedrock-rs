use bedrock_protocol_version::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct DataItem<V: ProtoVersion> {
    pub data_item_id: V::ActorDataIDs,
    pub data_item_type: V::DataItemType,
}
