use crate::version::versions::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 187)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateAbilitiesPacket<V: ProtoVersion> {
    pub data: V::SerializedAbilitiesData,
}
