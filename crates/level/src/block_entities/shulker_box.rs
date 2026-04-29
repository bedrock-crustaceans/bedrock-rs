use crate::block_entities::Chest;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ShulkerBox {
    #[serde(flatten)]
    contents: Chest,
    facing: f32,
}
