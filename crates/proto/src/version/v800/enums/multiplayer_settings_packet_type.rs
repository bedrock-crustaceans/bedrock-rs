use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum MultiplayerSettingsPacketType {
    EnableMultiplayer = 0,
    DisableMultiplayer = 1,
    RefreshJoinCode = 2,
}