use crate::block_entities::ItemStack;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Campfire {
    /// The item in position 1.
    pub item1: Option<ItemStack>,
    /// The item in position 2.
    pub item2: Option<ItemStack>,
    /// The item in position 3.
    pub item3: Option<ItemStack>,
    /// The item in position 4.
    pub item4: Option<ItemStack>,

    /// How long item 1 has been cooking.
    pub item_time1: i32,
    /// How long item 2 has been cooking.
    pub item_time2: i32,
    /// How long item 3 has been cooking.
    pub item_time3: i32,
    /// How long item 4 has been cooking.
    pub item_time4: i32,
}
