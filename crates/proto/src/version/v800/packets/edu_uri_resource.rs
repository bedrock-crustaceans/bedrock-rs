use super::super::types::EduSharedUriResource;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 170)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct EduUriResourcePacket {
    pub edu_shared_uri_resource: EduSharedUriResource,
}