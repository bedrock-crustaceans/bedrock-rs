use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(le)]
#[repr(i32)]
pub enum BuildPlatform {
    Google = 1,
    IOS = 2,
    OSX = 3,
    Amazon = 4,
    GearVR = 5,
    UWP = 7,
    Win32 = 8,
    Dedicated = 9,
    #[deprecated]
    TvOs = 10,
    Sony = 11,
    /// PlayStation
    Nx = 12,
    /// Nintendo Switch
    Xbox = 13,
    #[deprecated]
    WindowsPhone = 14,
    Linux = 15,
    #[enum_fallback]
    Unknown(i32),
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_protocol_core::ProtoCodec;
    use std::io::Cursor;

    #[test]
    fn unlisted_build_platform_survives_a_round_trip() {
        let bytes = 42i32.to_le_bytes();
        let platform = BuildPlatform::deserialize(&mut Cursor::new(&bytes[..])).unwrap();
        assert!(matches!(platform, BuildPlatform::Unknown(42)));
        let mut encoded = Vec::new();
        platform.serialize(&mut encoded).unwrap();
        assert_eq!(encoded, bytes);
    }
}
