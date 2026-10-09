use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 355)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundStonecutterSetRecipePacket<V: ProtoVersion> {
    pub actor_id: V::ActorUniqueID,
    pub container_id: u8,
    #[endianness(var)]
    pub recipe_index: i32,
}
