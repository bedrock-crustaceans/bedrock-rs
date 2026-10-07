use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 354)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerBoundStonecutterSetRecipePacket {
    pub container_id: u8,
    #[endianness(var)]
    pub recipe_index: i32,
}
