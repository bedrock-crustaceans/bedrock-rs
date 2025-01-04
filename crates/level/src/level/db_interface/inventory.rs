use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct InventoryData {
    pub items: Vec<InventoryItem>, 
}

#[derive(Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: u32,
    pub quantity: u32, 
}
