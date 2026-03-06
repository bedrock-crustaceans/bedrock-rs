use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 35)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ActorPickRequestPacket {
    #[endianness(le)]
    pub actor_id: i64,
    pub max_slots: i8,
    pub with_data: bool,
}
