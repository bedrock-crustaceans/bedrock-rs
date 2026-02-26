use super::super::types::SerializedAbilitiesData;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 187)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateAbilitiesPacket {
    pub data: SerializedAbilitiesData,
}