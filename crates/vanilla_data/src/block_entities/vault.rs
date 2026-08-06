use bedrock_level::types::ItemStack;
use facet::Facet;

#[derive(Debug, Clone, PartialEq, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct VaultData {
    pub items_to_eject: Vec<ItemStack>,
    pub rewarded_players: Vec<i64>,
    pub state_updating_resumes_at: i64,
}

/// The item a vault accepts, stored as a `String` tag holding the item id.
///
/// The hand-written serde impls this replaces did exactly what
/// `variant_as(str)` plus a per-variant rename does: match the id, reject
/// anything else.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Facet)]
#[facet(nbtx::variant_as(str))]
#[repr(u8)]
pub enum VaultKeyType {
    #[facet(rename = "minecraft:trial_key")]
    Normal,
    #[facet(rename = "minecraft:ominous_trial_key")]
    Ominous,
}

#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct VaultKeyItem {
    #[facet(rename = "Name")]
    pub ty: VaultKeyType,
    pub damage: i16,
    pub count: i8,
    pub was_picked_up: bool,
}

#[derive(Debug, Clone, PartialEq, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct VaultConfig {
    pub loot_table: String,
    pub activation_range: f32,
    pub key_item: VaultKeyItem,
    pub deactivation_range: f32,
    pub override_loot_table_to_display: String,
}

#[derive(Debug, Clone, PartialEq, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct Vault {
    pub data: VaultData,
    pub config: VaultConfig,
}
