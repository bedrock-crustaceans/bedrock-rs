/// A noteblock tile entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Noteblock {
    /// The pitch of the note block.
    pub note: i8,
}
