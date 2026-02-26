use super::super::enums::SimulationType;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 168)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SimulationTypePacket {
    pub sim_type: SimulationType,
}