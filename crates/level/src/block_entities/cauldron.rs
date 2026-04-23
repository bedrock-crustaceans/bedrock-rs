pub struct CauldronItem {}

pub struct Cauldron {
    pub custom_color: Option<i32>,
    pub items: Vec<CauldronItem>,
    pub potion_id: i16,
    pub potion_type: i16,
}
