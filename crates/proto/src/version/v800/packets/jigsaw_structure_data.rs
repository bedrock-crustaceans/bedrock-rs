use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 313)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct JigsawStructureDataPacket {
    #[nbt]
    jigsaw_structure_data_tag: nbtx::Value,
}