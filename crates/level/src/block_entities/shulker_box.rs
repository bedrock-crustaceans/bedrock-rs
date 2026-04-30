use crate::block_entities::Chest;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct ShulkerBox {
    #[serde(flatten)]
    contents: Chest,
    facing: f32,
}
