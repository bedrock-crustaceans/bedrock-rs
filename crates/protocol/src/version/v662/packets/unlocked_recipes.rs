use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 199, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UnlockedRecipesPacket {
    #[endianness(le)]
    pub packet_type: u32,
    pub unlocked_recipes_list: Vec<String>,
}
