use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 139)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MultiplayerSettingsPacket<V: ProtoVersion> {
    pub multiplayer_settings_packet_type: V::MultiplayerSettingsPacketType,
}
