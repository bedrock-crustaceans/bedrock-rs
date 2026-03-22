use bedrockrs_macros::{ProtoCodec, packet};

#[packet(id = 333)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundDataDrivenUIShowScreenPacket {
    pub screen_id: String,
    #[endianness(le)]
    pub form_id: i32,
    #[endianness(le)]
    pub data_instance_id: Option<i32>
}