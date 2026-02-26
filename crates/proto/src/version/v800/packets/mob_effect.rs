use super::super::types::ActorRuntimeID;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 28)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MobEffectPacket {
    pub target_runtime_id: ActorRuntimeID,
    pub event_id: Event,
    #[endianness(var)]
    pub effect_id: i32,
    #[endianness(var)]
    pub effect_amplifier: i32,
    pub show_particles: bool,
    #[endianness(var)]
    pub effect_duration_ticks: i32,
    #[endianness(var)]
    pub tick: u64,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum Event {
    Invalid = 0,
    Add = 1,
    Update = 2,
    Remove = 3,
}
