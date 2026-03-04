use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 168)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SimulationTypePacket<V: ProtoVersion> {
    pub sim_type: V::SimulationType,
}