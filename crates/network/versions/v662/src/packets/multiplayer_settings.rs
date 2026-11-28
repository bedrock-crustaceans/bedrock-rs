use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 139)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MultiplayerSettingsPacket<V: ProtoVersion> {
    pub multiplayer_settings_packet_type: V::MultiplayerSettingsPacketType,
}
