use super::super::enums::MultiplayerSettingsPacketType;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 139)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MultiplayerSettingsPacket {
    pub multiplayer_settings_packet_type: MultiplayerSettingsPacketType,
}