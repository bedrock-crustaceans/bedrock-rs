use bedrock_network_macros::{ProtoCodec, packet};
use bedrock_network_version_v924::packets::ClientBoundDataStoreValue;

#[packet(id = 330, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundDataStorePacket {
    pub updates: Vec<ClientBoundDataStoreUpdate>,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum ClientBoundDataStoreUpdate {
    Update {
        variable_name: String,
        data: ClientBoundDataStoreValue,
        #[endianness(le)]
        update_count: u32,
    } = 0,
    Change {
        variable_name: String,
        #[endianness(le)]
        update_count: u32,
        new_value: DynamicValue,
    } = 1,
    Remove {
        variable_name: String,
    } = 2,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(le)]
#[repr(u32)]
pub enum DynamicValue {
    Null = 0,
    Bool(bool) = 1,
    Long(#[endianness(le)] i64) = 2,
    Double(#[endianness(le)] f64) = 3,
    String(String) = 4,
    List(Vec<DynamicValue>) = 5,
    Map(Vec<DynamicValueMapEntry>) = 6,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct DynamicValueMapEntry {
    pub key: String,
    pub value: DynamicValue,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_network_core::ProtoCodec;
    use std::io::Cursor;

    #[test]
    fn data_store_change_carries_a_nested_dynamic_value() {
        let bytes = [
            0x01, 0x01, 0x01, b'a', 0x02, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x01,
            b'k', 0x05, 0x00, 0x00, 0x00, 0x02, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x00,
        ];
        let packet = ClientBoundDataStorePacket::deserialize(&mut Cursor::new(&bytes[..])).unwrap();
        let mut encoded = Vec::new();
        packet.serialize(&mut encoded).unwrap();
        assert_eq!(encoded, bytes, "encoding differs from the input");

        let [ClientBoundDataStoreUpdate::Change { variable_name, update_count, new_value }] =
            &packet.updates[..]
        else {
            panic!("expected one change, got {:?}", packet.updates);
        };
        assert_eq!(variable_name, "a");
        assert_eq!(*update_count, 2);
        let DynamicValue::Map(entries) = new_value else {
            panic!("expected a map, got {new_value:?}");
        };
        assert_eq!(entries[0].key, "k");
        assert!(matches!(
            &entries[0].value,
            DynamicValue::List(items)
                if matches!(items[..], [DynamicValue::Bool(true), DynamicValue::Null])
        ));
    }
}
