use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 158)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AnimateEntityPacket<V: ProtoVersion> {
    pub animation: String,
    pub next_state: String,
    pub stop_expression: String,
    pub stop_expression_molang_version: V::MolangVersion,
    pub controller: String,
    #[endianness(le)]
    pub blend_out_time: f32,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub runtime_ids: Vec<V::ActorRuntimeID>,
}
