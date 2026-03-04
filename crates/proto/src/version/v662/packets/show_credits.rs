use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 75)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ShowCreditsPacket<V: ProtoVersion> {
    pub player_runtime_id: V::ActorRuntimeID,
    pub credits_state: CreditsState,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum CreditsState {
    Start = 0,
    Finished = 1,
}