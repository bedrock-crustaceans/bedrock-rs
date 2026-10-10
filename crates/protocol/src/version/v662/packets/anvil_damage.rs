use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 141, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AnvilDamagePacket<V: ProtoVersion> {
    pub damage_amount: i8,
    pub block_position: V::NetworkBlockPosition,
}
