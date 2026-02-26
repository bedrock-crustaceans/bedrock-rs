use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 106)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RemoveObjectivePacket {
    pub objective_name: String,
}