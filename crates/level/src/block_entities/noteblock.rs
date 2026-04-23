/// A noteblock tile entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Noteblock {
    /// The pitch of the note block.
    note: i8,
}
