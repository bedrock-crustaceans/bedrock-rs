use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 85)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct TransferPlayerPacket<V: ProtoVersion> {
    pub server_address: String,
    #[endianness(le)]
    pub server_port: u16,
    /// Whether the client should reload the world on transfer.
    pub reload_world: bool,
    pub gatherings_config: Option<V::GatheringsConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::V2168;
    use bedrock_protocol_core::ProtoCodec;

    #[test]
    fn matches_gophertunnel_wire_layout() {
        let packet = TransferPlayerPacket::<V2168> {
            server_address: "1.2.3.4".into(),
            server_port: 19132,
            reload_world: false,
            gatherings_config: None,
        };
        let mut encoded = Vec::new();
        packet.serialize(&mut encoded).unwrap();

        // gophertunnel writes Address, Port, ReloadWorld, then GatheringJoinInfo.
        assert_eq!(
            encoded,
            [
                &[0x07][..],
                b"1.2.3.4",
                &[0xbc, 0x4a],
                &[0x00],
                &[0x00],
            ]
            .concat()
        );
    }
}
