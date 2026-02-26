use super::super::types::NetworkBlockPosition;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 141)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AnvilDamagePacket {
    pub damage_amount: i8,
    pub block_position: NetworkBlockPosition,
}