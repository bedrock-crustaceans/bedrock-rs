// TODO: Use potion ID enum instead of integers.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Beacon {
    pub primary: i32,
    pub secondary: i32,
}
