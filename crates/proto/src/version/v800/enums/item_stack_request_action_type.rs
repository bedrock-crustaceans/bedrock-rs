use bedrockrs_macros::ProtoCodec;
use super::super::enums::ItemDescriptorType;
use super::super::types::ItemStackRequestSlotInfo;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum ItemStackRequestActionType {
    Take {
        amount: i8,
        source: ItemStackRequestSlotInfo,
        destination: ItemStackRequestSlotInfo,
    } = 0,
    Place {
        amount: i8,
        source: ItemStackRequestSlotInfo,
        destination: ItemStackRequestSlotInfo,
    } = 1,
    Swap {
        source: ItemStackRequestSlotInfo,
        destination: ItemStackRequestSlotInfo,
    } = 2,
    Drop {
        amount: i8,
        source: ItemStackRequestSlotInfo,
        randomly: bool,
    } = 3,
    Destroy {
        amount: i8,
        source: ItemStackRequestSlotInfo,
    } = 4,
    Consume {
        amount: i8,
        source: ItemStackRequestSlotInfo,
    } = 5,
    Create {
        slot: i8,
    } = 6,
    #[deprecated]
    PlaceInItemContainer = 7,
    #[deprecated]
    TakeFromItemContainer = 8,
    ScreenLabTableCombine = 9,
    ScreenBeaconPayment {
        #[endianness(var)]
        primary_effect: i32,
        #[endianness(var)]
        secondary_effect: i32,
    } = 10,
    ScreenHUDMineBlock {
        #[endianness(var)]
        hotbar_slot: i32,
        #[endianness(var)]
        predicted_durability: i32,
        #[endianness(var)]
        stack_network_id: i32,
    } = 11,
    CraftRecipe {
        #[endianness(var)]
        recipe_network_id: i32,
        number_of_requested_crafts: i8,
    } = 12,
    CraftRecipeAuto {
        #[endianness(var)]
        recipe_network_id: i32,
        number_of_requested_crafts: i8,
        times_crafted: i8,
        #[vec_repr(u8)]
        ingredients: Vec<ItemDescriptorType>
    } = 13,
    CraftCreative {
        #[endianness(var)]
        creative_item_network_id: i32,
        number_of_requested_crafts: i8,
    } = 14,
    CraftRecipeOptional {
        #[endianness(var)]
        recipe_network_id: i32,
        #[endianness(le)]
        filtered_strings_index: i32,
    } = 15,
    CraftRepairAndDisenchant {
        #[endianness(var)]
        recipe_network_id: i32,
        number_of_requested_crafts: i8,
        #[endianness(var)]
        repair_cost: i32,
    } = 16,
    CraftLoom {
        pattern_id: String,
        number_of_requested_crafts: i8,
    } = 17,
    #[deprecated = "Ask Tylaing"] CraftNonImplemented = 18,
    #[deprecated = "Ask Tylaing"] CraftResults {
        // TODO: 
        // #[vec_repr(i32)]
        // #[vec_endianness(var)]
        // result_items: Vec<Item>
        // times_crafted: i8,
    } = 19,
}