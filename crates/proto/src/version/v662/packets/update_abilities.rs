use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 187)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateAbilitiesPacket<V: ProtoVersion> {
    pub data: V::SerializedAbilitiesData,
}
