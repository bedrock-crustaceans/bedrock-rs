use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 313)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct JigsawStructureDataPacket {
    #[nbt]
    jigsaw_structure_data_tag: nbtx::Value,
}
