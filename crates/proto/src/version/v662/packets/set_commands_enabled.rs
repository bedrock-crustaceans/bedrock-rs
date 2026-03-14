use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 59)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetCommandsEnabledPacket {
    pub commands_enabled: bool,
}
