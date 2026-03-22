use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum ActorBlockSyncMessageID {
    None = 0,
    Create = 1,
    Destroy = 2,
}
