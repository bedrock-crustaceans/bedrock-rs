use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug, PartialEq)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum InventorySourceType {
    InvalidInventory = u32::MAX,
    ContainerInventory(#[endianness(var)] i32) = 0,
    GlobalInventory = 1,
    WorldInteraction(#[endianness(var)] u32) = 2,
    CreativeInventory = 3,
    UntrackedInteractionUI(#[endianness(var)] i32) = 100,
    NonImplementedFeatureTODO(#[endianness(var)] i32) = 99999,
}
