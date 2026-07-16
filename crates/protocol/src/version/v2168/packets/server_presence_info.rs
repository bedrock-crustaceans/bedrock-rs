use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 347)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerPresenceInfoPacket {
    pub presence_configuration: Option<PresenceConfiguration>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct PresenceConfiguration {
    pub rich_presence_id: Option<String>,
}
