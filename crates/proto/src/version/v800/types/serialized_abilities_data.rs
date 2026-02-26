use super::super::enums::{CommandPermissionLevel, PlayerPermissionLevel};
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct SerializedAbilitiesData {
    #[endianness(le)]
    pub target_player_raw_id: i64,
    pub player_permissions: PlayerPermissionLevel,
    pub command_permissions: CommandPermissionLevel,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub layers: Vec<SerializedLayer>,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u16)]
#[enum_endianness(le)]
#[repr(u16)]
pub enum SerializedAbilitiesLayer {
    CustomCache = 0,
    Base = 1,
    Spectator = 2,
    Commands = 3,
    Editor = 4,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct SerializedLayer {
    pub serialized_layer: SerializedAbilitiesLayer,
    #[endianness(le)]
    pub abilities_set: u32,
    #[endianness(le)]
    pub ability_values: u32,
    #[endianness(le)]
    pub fly_speed: f32,
    #[endianness(le)]
    pub walk_speed: f32,
}

