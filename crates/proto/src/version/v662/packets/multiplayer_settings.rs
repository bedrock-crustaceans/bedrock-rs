use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 139)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MultiplayerSettingsPacket<V: ProtoVersion> {
    pub multiplayer_settings_packet_type: V::MultiplayerSettingsPacketType,
}