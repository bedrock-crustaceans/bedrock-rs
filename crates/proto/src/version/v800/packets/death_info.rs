use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 189)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct DeathInfoPacket {
    pub death_cause_attack_name: String,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub death_cause_message_list: Vec<String>,
}