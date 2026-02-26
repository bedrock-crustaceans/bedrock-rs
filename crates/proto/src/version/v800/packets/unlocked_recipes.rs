use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 199)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UnlockedRecipesPacket {
    #[endianness(le)]
    pub packet_type: u32,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub unlocked_recipes_list: Vec<String>,
}