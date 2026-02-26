use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::v671::types::ActorUniqueID;

#[gamepacket(id = 325)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerUpdateEntityOverridesPacket {
    pub entity_unique_id: ActorUniqueID,
    #[endianness(var)]
    pub property_index: u32,
    pub update_type: UpdateType,
    
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum UpdateType {
    ClearOverrides = 0,
    RemoveOverride = 1,
    SetIntOverride {
        #[endianness(le)]
        value: i32
    } = 2,
    SetFloatOverride {
        #[endianness(le)]
        value: f32
    } = 3,
}