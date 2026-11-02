//! The published script `metadata/` files as written, one module per file family.
//! Each file type has a strict `parse(&[u8])`: the files are written
//! by the game's metadata generator, so a key no field takes is an error, not a member to keep.
//! Nothing here is read at pack-load time; the generated tables and chorus's code generator are
//! built from these models.

pub mod commands;
pub mod engine;
pub mod molang;
pub mod script;
pub mod vanilla;

/// Why a scalar whose JSON type varies (a constant's value, a block-state value) was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ShapeError {
    /// The JSON type is none of those the member takes.
    #[error("expected {expected}")]
    WrongType {
        /// What the member takes, in words.
        expected: &'static str,
    },
    /// An integer outside the member's range.
    #[error("the integer does not fit")]
    OutOfRange,
}
