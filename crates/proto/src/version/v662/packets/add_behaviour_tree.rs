use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 89)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AddBehaviourTreePacket {
    pub json_behaviour_tree_structure: String,
}
