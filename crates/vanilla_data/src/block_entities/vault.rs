use bedrock_level::serde_helpers::deserialize_bool;
use bedrock_level::types::ItemStack;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct VaultData {
    pub items_to_eject: Vec<ItemStack>,
    pub rewarded_players: Vec<i64>,
    pub state_updating_resumes_at: i64,
}

const TRIAL_KEY_NORMAL: &str = "minecraft:trial_key";
const TRIAL_KEY_OMINOUS: &str = "minecraft:ominous_trial_key";

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VaultKeyType {
    Normal,
    Ominous,
}

impl serde::Serialize for VaultKeyType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let v = match self {
            Self::Normal => TRIAL_KEY_NORMAL,
            Self::Ominous => TRIAL_KEY_OMINOUS,
        };

        v.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for VaultKeyType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = String::deserialize(deserializer)?;

        Ok(match v.as_str() {
            TRIAL_KEY_NORMAL => Self::Normal,
            TRIAL_KEY_OMINOUS => Self::Ominous,
            _ => {
                return Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(&v),
                    &"string `minecraft:trial_key` or `minecraft:ominous_trial_key`",
                ));
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct VaultKeyItem {
    #[serde(rename = "Name")]
    pub ty: VaultKeyType,
    pub damage: i16,
    pub count: i8,
    #[serde(deserialize_with = "deserialize_bool")]
    pub was_picked_up: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct VaultConfig {
    pub loot_table: String,
    pub activation_range: f32,
    pub key_item: VaultKeyItem,
    pub deactivation_range: f32,
    pub override_loot_table_to_display: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Vault {
    pub data: VaultData,
    pub config: VaultConfig,
}
