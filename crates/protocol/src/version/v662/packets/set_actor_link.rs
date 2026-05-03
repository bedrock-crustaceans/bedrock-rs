use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 41)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetActorLinkPacket<V: ProtoVersion> {
    pub link: V::ActorLink,
}
