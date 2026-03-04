use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u64)]
#[enum_endianness(var)]
#[repr(u64)]
pub enum ActorBlockSyncMessageID {
    None = 0,
    Create = 1,
    Destroy = 2,
}
