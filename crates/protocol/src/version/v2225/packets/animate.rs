use crate::ProtoVersion;
use crate::version::v898::packets::{AnimatePacketAction, SwingSource};
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 44)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AnimatePacket<V: ProtoVersion> {
    pub action: AnimatePacketAction,
    pub target_runtime_id: V::ActorRuntimeID,
    #[endianness(le)]
    pub data: f32,
    pub swing_source: Option<SwingSource>,
    pub hand: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_protocol_core::ProtoCodec;
    use std::io::Cursor;

    #[test]
    fn animate_ends_with_the_hand_slot() {
        let bytes = [1, 5, 0, 0, 0, 0, 1, 6, b'a', b't', b't', b'a', b'c', b'k', 1];
        let packet = AnimatePacket::<crate::V2225>::deserialize(&mut Cursor::new(&bytes[..])).unwrap();
        let mut encoded = Vec::new();
        packet.serialize(&mut encoded).unwrap();
        assert_eq!(encoded, bytes, "encoding differs from the input");
        assert_eq!(packet.hand, 1);
        assert!(matches!(packet.swing_source, Some(SwingSource::Attack)));
    }
}
