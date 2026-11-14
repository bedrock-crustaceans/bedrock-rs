use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 342, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PartyChangedPacket {
    pub party_info: Option<PlayerPartyInfo>
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerPartyInfo {
    pub party_id: String,
    pub is_party_leader: bool
}