use super::super::enums::PlayerPermissionLevel;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 185)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RequestPermissionsPacket {
    #[endianness(le)]
    pub target_player_raw_id: i64,
    pub player_permission_level: PlayerPermissionLevel,
    #[endianness(le)]
    pub custom_permission_flags: u16,
}