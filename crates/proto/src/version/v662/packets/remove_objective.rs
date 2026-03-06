use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 106)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RemoveObjectivePacket {
    pub objective_name: String,
}
