use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 328)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct DebugDrawerPacket<V: ProtoVersion> {
    #[vec_repr(i32)]
    #[vec_endianness(var)]
    pub shapes: Vec<V::DebugShape>,
}
