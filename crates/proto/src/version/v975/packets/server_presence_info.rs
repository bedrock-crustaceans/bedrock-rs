use bedrockrs_macros::{ProtoCodec, packet};

#[packet(id = 347)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerPresenceInfoPacket {
    pub presence_configuration: Option<PresenceConfiguration>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct PresenceConfiguration {
    pub experience_name: String,
    pub world_name: String
}