use super::super::types::AdventureSettings;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 188)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateAdventureSettingsPacket {
    pub adventure_settings: AdventureSettings,
}