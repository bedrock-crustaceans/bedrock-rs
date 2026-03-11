use crate::version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, packet};

#[packet(id = 28)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MobEffectPacket<V: ProtoVersion> {
    pub target_runtime_id: V::ActorRuntimeID,
    pub event_id: MobEffectEvent,
    #[endianness(var)]
    pub effect_id: i32,
    #[endianness(var)]
    pub effect_amplifier: i32,
    pub show_particles: bool,
    #[endianness(var)]
    pub effect_duration_ticks: i32,
    #[endianness(var)]
    pub tick: u64,
    pub ambient: bool,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum MobEffectEvent {
    Invalid = 0,
    Add = 1,
    Update = 2,
    Remove = 3,
}
