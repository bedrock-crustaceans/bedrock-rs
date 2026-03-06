use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 75)]
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
