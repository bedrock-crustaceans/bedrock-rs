use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 188)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateAdventureSettingsPacket<V: ProtoVersion> {
    pub adventure_settings: V::AdventureSettings,
}
