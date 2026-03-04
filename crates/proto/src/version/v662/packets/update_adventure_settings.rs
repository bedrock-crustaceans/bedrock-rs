use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 188)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateAdventureSettingsPacket<V: ProtoVersion> {
    pub adventure_settings: V::AdventureSettings,
}