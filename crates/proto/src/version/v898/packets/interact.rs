use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};
use bedrockrs_proto_core::ProtoCodec;
use bedrockrs_proto_core::error::ProtoCodecError;
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};
use vek::Vec3;

#[gamepacket(id = 33)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct InteractPacket<V: ProtoVersion> {
    pub action: InteractPacketAction,
    pub target_runtime_id: V::ActorRuntimeID,
    #[endianness(le)]
    pub mouse_position: Option<Vec3<f32>>,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum InteractPacketAction {
    Invalid = 0,
    Interact = 1,
    Damage = 2,
    StopRiding = 3,
    InteractUpdate = 4,
    NpcOpen = 5,
    OpenInventory = 6,
}
