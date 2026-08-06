use facet::Facet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct SculkCatalystCursor {
    pub charge: i16,
    pub decay: i16,
    pub facing: i16,
    pub update: i16,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct SculkCatalyst {
    pub cursors: Option<Vec<SculkCatalystCursor>>,
}
