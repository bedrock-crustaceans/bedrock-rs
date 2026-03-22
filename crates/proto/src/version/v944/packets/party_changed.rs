use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 342)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PartyChangedPacket {
    pub party_id: String
}