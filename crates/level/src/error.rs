use std::sync::PoisonError;

use bedrock_shared::world::dimension::Dimension;
use thiserror::Error;

use crate::types::ChunkPosition;

#[derive(Debug, Error)]
pub enum Error {
    #[error("requested {requested} bits for array, but requires at least {required}")]
    TooFewBits { requested: u32, required: u32 },
    #[error("failed to acquire database lock")]
    DatabaseLockError,
    #[error("leveldb error: {0}")]
    LevelDbError(String),
    #[error("{0}")]
    NbtError(#[from] nbtx::Error),
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("invalid packed array index size: {0}")]
    InvalidBitSize(u8),
    #[error("invalid {0}")]
    Invalid(&'static str),
    #[error("unknown error")]
    Unknown,
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_err: PoisonError<T>) -> Error {
        Error::DatabaseLockError
    }
}

impl From<leveldb::Status> for Error {
    fn from(err: leveldb::Status) -> Error {
        Error::LevelDbError(err.err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// A per-item error from a bulk chunk walk (see [`crate::level::Chunks`]), naming which
/// chunk failed rather than leaving the caller to guess -- every record group in the walk
/// is read up front in one database scan, but decoding is deferred one chunk at a time as
/// the iterator is driven, and each item is a `Result` (architecture decision 11), so a bare
/// [`Error`] on its own would tell a caller skipping bad chunks *that* one failed without
/// saying which. A wrapper at the iterator boundary rather than position fields threaded
/// into every [`Error`] variant: most `Error`s are raised well inside decode, with no chunk
/// coordinates in scope at all (a standalone `SubChunk::from_disk`, an NBT decode inside
/// `Chunk::decode` called directly, outside any `Level`), so the position is a property of
/// *this walk*, not of the error itself.
#[derive(Debug, Error)]
pub struct ChunkError {
    /// The position of the chunk that failed to decode.
    ///
    /// Ordered before `dimension` deliberately: this matches both this type's own `Display`
    /// message ("chunk at (x, z) in <dimension>: ...") and [`crate::key::Key`]'s field order
    /// (`chunk` before `dimension`), the crate's other position/dimension-bearing struct --
    /// not the `dimension`-before-`position` order `Level`'s methods take as parameters,
    /// which is a call-site convention rather than a struct-layout one.
    pub position: ChunkPosition,
    /// The dimension the chunk that failed to decode belongs to.
    pub dimension: Dimension,
    /// The underlying decode failure.
    #[source]
    pub source: Error,
}

impl std::fmt::Display for ChunkError {
    /// `ChunkPosition`'s own `Debug` reads as `ChunkPosition(x, z)`, which repeats the type
    /// name for no benefit in a message that already says "chunk"; this spells the pair out
    /// as `(x, z)` instead. `dimension` uses `Debug`, which for this enum already reads as a
    /// bare variant name (`Overworld`, `Other(5)`, ...), so it needs no separate formatting.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "chunk at ({}, {}) in {:?}: {}",
            self.position.0, self.position.1, self.dimension, self.source
        )
    }
}
