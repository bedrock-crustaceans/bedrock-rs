use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 141)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AnvilDamagePacket<V: ProtoVersion> {
    pub damage_amount: i8,
    pub block_position: V::NetworkBlockPosition,
}
