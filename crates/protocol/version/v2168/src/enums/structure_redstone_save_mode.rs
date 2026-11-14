use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u8)]
#[repr(u8)]
pub enum StructureRedstoneSaveMode {
    SavesToMemory = 0,
    SavesToDisk = 1,
}
