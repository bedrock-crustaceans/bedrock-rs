use bedrockrs_macros::{ProtoCodec, gamepacket};
use uuid::Uuid;

#[gamepacket(id = 329)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerBoundPackSettingChangePacket {
    pub pack_id: Uuid,
    pub pack_setting: PackSetting,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct PackSetting {
    pub name: String,
    pub value: PackSettingValue,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum PackSettingValue {
    Float(#[endianness(le)] f32) = 0,
    Bool(bool) = 1,
    String(String) = 2,
}
