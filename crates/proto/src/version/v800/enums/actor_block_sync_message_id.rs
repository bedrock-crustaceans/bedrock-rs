use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u64)]
#[enum_endianness(var)]
#[repr(u64)]
pub enum ActorBlockSyncMessageID {
    NONE = 0,
    CREATE = 1,
    DESTROY = 2,
}

