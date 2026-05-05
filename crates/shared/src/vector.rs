#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct Position(pub f32, pub f32, pub f32);

impl From<(f32, f32, f32)> for Position {
    fn from((x, y, z): (f32, f32, f32)) -> Self {
        Self(x, y, z)
    }
}

impl From<[f32; 3]> for Position {
    fn from([x, y, z]: [f32; 3]) -> Self {
        Self(x, y, z)
    }
}

/// Rotation of an entity.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct Rotation(pub f32, pub f32);

impl From<(f32, f32)> for Rotation {
    fn from((x, y): (f32, f32)) -> Self {
        Self(x, y)
    }
}

impl From<[f32; 2]> for Rotation {
    fn from([x, y]: [f32; 2]) -> Self {
        Self(x, y)
    }
}
