use super::super::enums::ControlScheme;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 327)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundControlSchemeSetPacket {
    pub scheme: ControlScheme,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrockrs_proto_core::ProtoCodec as _;
    use std::io::Cursor;

    #[test]
    fn roundtrip_client_bound_control_scheme_set() {
        let packet = ClientBoundControlSchemeSetPacket {
            scheme: ControlScheme::CameraRelativeStrafe,
        };

        let mut buf = Vec::new();
        packet.proto_serialize(&mut buf).unwrap();

        let mut cur = Cursor::new(buf.as_slice());
        let decoded = ClientBoundControlSchemeSetPacket::proto_deserialize(&mut cur).unwrap();
        assert_eq!(decoded.scheme, ControlScheme::CameraRelativeStrafe);
    }
}
