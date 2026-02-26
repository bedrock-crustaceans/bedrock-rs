use bedrockrs_macros::ProtoCodec;

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
    #[deprecated] TvOs = 10,
    Sony = 11,
    /// PlayStation
    Nx = 12,
    /// Nintendo Switch
    Xbox = 13,
    #[deprecated] WindowsPhone = 14,
    Linux = 15,
    Unknown = -1,
}