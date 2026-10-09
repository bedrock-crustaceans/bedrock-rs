use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 348)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundUpdateSoundDataPacket<V: ProtoVersion> {
    #[endianness(le)]
    pub server_sound_handle: i64,
    pub event: V::SoundData,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::v2225::types::SoundData;
    use bedrock_protocol_core::ProtoCodec;
    use std::io::Cursor;

    fn round_trip(bytes: &[u8]) -> ClientBoundUpdateSoundDataPacket<crate::V2225> {
        let packet =
            ClientBoundUpdateSoundDataPacket::<crate::V2225>::deserialize(&mut Cursor::new(bytes))
                .unwrap();
        let mut encoded = Vec::new();
        packet.serialize(&mut encoded).unwrap();
        assert_eq!(encoded, bytes, "encoding differs from the input");
        packet
    }

    #[test]
    fn update_sound_data_carries_a_single_event() {
        let packet = round_trip(&[7, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0x3f, 0, 0, 0x80, 0x3f]);
        assert_eq!(packet.server_sound_handle, 7);
        assert!(matches!(
            packet.event,
            SoundData::Fade { duration, target_volume } if duration == 0.5 && target_volume == 1.0
        ));
    }
}
