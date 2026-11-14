use bedrock_protocol_version::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 168, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SimulationTypePacket<V: ProtoVersion> {
    pub sim_type: V::SimulationType,
}
