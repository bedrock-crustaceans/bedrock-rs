use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::ProtoCodec;
use bedrock_network_version_v2225::types::PassengerOfBlockEmoteType;
use uuid::Uuid;

#[derive(ProtoCodec, Clone, Debug)]
pub struct PassengerOfBlockArguments<V: ProtoVersion> {
    pub block_position: V::NetworkBlockPosition,
    #[endianness(le)]
    pub offset: (f32, f32, f32),
    #[endianness(le)]
    pub rotation: f32,
    #[endianness(le)]
    pub rotation_limit: f32,
    pub emote_type: PassengerOfBlockEmoteType,
    #[endianness(le)]
    pub block_type: u64,
    pub pack_uuid: Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_network_core::ProtoCodec;
    use std::io::Cursor;

    #[test]
    fn passenger_of_block_arguments_end_with_block_type_and_pack_uuid() {
        let mut bytes = vec![0x02, 0x04, 0x06];
        bytes.extend_from_slice(&[0; 12]);
        bytes.extend_from_slice(&1.5f32.to_le_bytes());
        bytes.extend_from_slice(&90f32.to_le_bytes());
        bytes.push(0x02);
        bytes.extend_from_slice(&0x0102_0304_0506_0708u64.to_le_bytes());
        bytes.extend_from_slice(&[0xAB; 16]);

        let value =
            PassengerOfBlockArguments::<crate::V2259>::deserialize(&mut Cursor::new(&bytes[..]))
                .unwrap();
        let mut encoded = Vec::new();
        value.serialize(&mut encoded).unwrap();
        assert_eq!(encoded, bytes, "encoding differs from the input");
        assert_eq!(value.block_type, 0x0102_0304_0506_0708);
        assert!(matches!(value.emote_type, PassengerOfBlockEmoteType::Laying));
    }
}
