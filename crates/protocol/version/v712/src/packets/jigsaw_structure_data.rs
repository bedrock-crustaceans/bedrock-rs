use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 313, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct JigsawStructureDataPacket {
    #[nbt]
    pub jigsaw_structure_data_tag: nbtx::Value,
}
