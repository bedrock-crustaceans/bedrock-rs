use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 170)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct EduUriResourcePacket<V: ProtoVersion> {
    pub edu_shared_uri_resource: V::EduSharedUriResource,
}