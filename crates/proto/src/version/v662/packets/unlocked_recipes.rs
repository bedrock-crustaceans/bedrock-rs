use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 199)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UnlockedRecipesPacket {
    #[endianness(le)]
    pub packet_type: u32,
    pub unlocked_recipes_list: Vec<String>,
}
