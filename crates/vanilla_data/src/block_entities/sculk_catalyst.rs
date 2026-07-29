#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
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
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct SculkCatalyst {
    pub cursors: Option<Vec<SculkCatalystCursor>>,
}
