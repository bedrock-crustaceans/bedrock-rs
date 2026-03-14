use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 168)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SimulationTypePacket<V: ProtoVersion> {
    pub sim_type: V::SimulationType,
}
