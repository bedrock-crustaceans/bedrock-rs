use bedrock_network_macros::{ProtoCodec, packet};

#[packet(id = 364, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerToggleArmorVisibilityRequestPacket {
    pub hide_armor: bool,
}
