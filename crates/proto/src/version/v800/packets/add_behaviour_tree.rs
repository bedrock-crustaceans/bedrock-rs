use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 89)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AddBehaviourTreePacket {
    pub json_behaviour_tree_structure: String,
}