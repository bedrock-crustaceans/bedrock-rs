use super::super::types::ActorRuntimeID;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 75)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ShowCreditsPacket {
    pub player_runtime_id: ActorRuntimeID,
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
