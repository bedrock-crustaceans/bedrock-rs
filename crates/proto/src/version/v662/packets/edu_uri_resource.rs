use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 170)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct EduUriResourcePacket<V: ProtoVersion> {
    pub edu_shared_uri_resource: V::EduSharedUriResource,
}
