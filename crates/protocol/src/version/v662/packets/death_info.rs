use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 189, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct DeathInfoPacket {
    pub death_cause_attack_name: String,
    pub death_cause_message_list: Vec<String>,
}
