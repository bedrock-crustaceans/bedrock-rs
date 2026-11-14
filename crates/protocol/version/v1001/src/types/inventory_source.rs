use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug, PartialEq)]
pub struct InventorySource {
    pub source_type: InventorySourceKind,
    pub container_id: Option<Option<i8>>,
    #[endianness(var)]
    pub flags: Option<Option<u32>>,
}

#[derive(ProtoCodec, Clone, Copy, Debug, PartialEq)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum InventorySourceKind {
    Invalid = u32::MAX,
    Container = 0,
    Global = 1,
    WorldInteraction = 2,
    Creative = 3,
    UntrackedInteractionUI = 100,
    NonImplementedFeatureTODO = 99999,
}
