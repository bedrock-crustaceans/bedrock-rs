use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 169)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct NpcDialoguePacket {
    #[endianness(le)]
    pub npc_raw_id: u64,
    pub npc_dialogue_action_type: NpcDialogueActionType,
    pub dialogue: String,
    pub scene_name: String,
    pub npc_name: String,
    pub action_json: String,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum NpcDialogueActionType {
    Open = 0,
    Close = 1,
}
