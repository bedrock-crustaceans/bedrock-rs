#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SculkCatalystCursor {
    pub charge: i16,
    pub decay: i16,
    pub facing: i16,
    pub update: i16,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SculkCatalyst {
    pub cursors: Option<Vec<SculkCatalystCursor>>,
}
