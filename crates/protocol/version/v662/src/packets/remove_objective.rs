use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 106, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RemoveObjectivePacket {
    pub objective_name: String,
}
