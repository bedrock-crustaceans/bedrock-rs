use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct DataItem<V: ProtoVersion> {
    pub data_item_id: V::ActorDataIDs,
    pub data_item_type: V::DataItemType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::v924::enums::ActorDataIDs;
    use bedrock_protocol_core::ProtoCodec;
    use std::io::Cursor;

    fn round_trip(bytes: &[u8]) -> DataItem<crate::V2225> {
        let item = DataItem::<crate::V2225>::deserialize(&mut Cursor::new(bytes)).unwrap();
        let mut encoded = Vec::new();
        item.serialize(&mut encoded).unwrap();
        assert_eq!(encoded, bytes, "encoding differs from the capture");
        item
    }

    #[test]
    fn data_item_key_is_a_named_actor_data_id() {
        let item = round_trip(&[0x04, 0x04, 0x04, 0x02, b'h', b'i']);
        assert!(matches!(item.data_item_id, ActorDataIDs::Name));
    }

    #[test]
    fn data_item_keeps_a_key_the_enum_does_not_know() {
        let item = round_trip(&[0xe7, 0x07, 0x00, 0x00, 0x01]);
        assert!(matches!(item.data_item_id, ActorDataIDs::Unknown(999)));
    }
}
