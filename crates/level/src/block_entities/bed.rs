use crate::color::Color;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Bed {
    /// Color of the bed.
    pub color: Color,
}
