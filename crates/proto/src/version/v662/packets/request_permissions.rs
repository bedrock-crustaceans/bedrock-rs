use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 185)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RequestPermissionsPacket<V: ProtoVersion> {
    #[endianness(le)]
    pub target_player_raw_id: i64,
    pub player_permission_level: V::PlayerPermissionLevel,
    #[endianness(le)]
    pub custom_permission_flags: u16,
}
