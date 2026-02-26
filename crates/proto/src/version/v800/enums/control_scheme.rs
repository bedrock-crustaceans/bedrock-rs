use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::ProtoCodec;
use std::io::Cursor;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ControlScheme {
    LockedPlayerRelativeStrafe = 0,
    CameraRelative = 1,
    CameraRelativeStrafe = 2,
    PlayerRelative = 3,
    PlayerRelativeStrafe = 4,
}

impl ProtoCodec for ControlScheme {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        <u8 as ProtoCodec>::proto_serialize(&(*self as u8), stream)
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let value = <u8 as ProtoCodec>::proto_deserialize(stream)?;
        match value {
            0 => Ok(Self::LockedPlayerRelativeStrafe),
            1 => Ok(Self::CameraRelative),
            2 => Ok(Self::CameraRelativeStrafe),
            3 => Ok(Self::PlayerRelative),
            4 => Ok(Self::PlayerRelativeStrafe),
            _ => Err(ProtoCodecError::InvalidEnumID(
                value.to_string(),
                "ControlScheme",
            )),
        }
    }

    fn get_size_prediction(&self) -> usize {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_scheme_roundtrip_each_variant() {
        let all = [
            ControlScheme::LockedPlayerRelativeStrafe,
            ControlScheme::CameraRelative,
            ControlScheme::CameraRelativeStrafe,
            ControlScheme::PlayerRelative,
            ControlScheme::PlayerRelativeStrafe,
        ];

        for value in all {
            let mut buf = Vec::new();
            value.proto_serialize(&mut buf).unwrap();

            let mut cur = Cursor::new(buf.as_slice());
            let decoded = ControlScheme::proto_deserialize(&mut cur).unwrap();
            assert_eq!(decoded, value);
        }
    }

    #[test]
    fn control_scheme_rejects_invalid_value() {
        let mut cur = Cursor::new(&[99u8][..]);
        let err = ControlScheme::proto_deserialize(&mut cur).unwrap_err();
        assert!(matches!(err, ProtoCodecError::InvalidEnumID(_, "ControlScheme")));
    }
}
