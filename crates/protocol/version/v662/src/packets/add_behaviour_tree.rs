use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 89, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AddBehaviourTreePacket {
    pub json_behaviour_tree_structure: String,
}
