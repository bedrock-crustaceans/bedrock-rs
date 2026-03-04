use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerBlockActions<V: ProtoVersion> {
    #[vec_repr(i32)]
    #[vec_endianness(var)]
    pub player_block_actions: Vec<V::PlayerBlockActionData>,
}
