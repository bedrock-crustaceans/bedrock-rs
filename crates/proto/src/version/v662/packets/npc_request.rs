use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 98)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct NpcRequestPacket<V: ProtoVersion> {
    pub npc_runtime_id: V::ActorRuntimeID,
    pub request_type: RequestType,
    pub actions: String,
    pub action_index: i8,
    pub scene_name: String,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum RequestType {
    SetActions = 0,
    ExecuteAction = 1,
    ExecuteClosingCommands = 2,
    SetName = 3,
    SetSkin = 4,
    SetInteractText = 5,
    ExecuteOpeningCommands = 6,
}