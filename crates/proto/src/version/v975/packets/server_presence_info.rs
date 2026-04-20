use bedrockrs_macros::{ProtoCodec, packet};
use crate::v975::types::PresenceConfiguration;

#[packet(id = 347)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerPresenceInfoPacket {
    pub presence_configuration: PresenceConfiguration,
}