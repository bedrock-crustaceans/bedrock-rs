// TODO: Use potion ID enum instead of integers.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Beacon {
    pub primary: i32,
    pub secondary: i32,
}
