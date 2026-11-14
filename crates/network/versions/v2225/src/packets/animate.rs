use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_version_v898::packets::{AnimatePacketAction, SwingSource};
use bedrock_network_macros::{ProtoCodec, packet};

#[packet(id = 44)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AnimatePacket<V: ProtoVersion> {
    pub action: AnimatePacketAction,
    pub target_runtime_id: V::ActorRuntimeID,
    #[endianness(le)]
    pub data: f32,
    pub swing_source: Option<SwingSource>,
    pub hand: u8,
}
