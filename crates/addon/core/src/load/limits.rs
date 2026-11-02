//! The bounds a loader enforces.

const MIB: u64 = 1024 * 1024;
const GIB: u64 = 1024 * MIB;

/// Bounds a loader enforces. `Default` gives the zip-bomb limits and the windows below.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Zip-bomb limit: 65 536 entries.
    pub max_entries: u32,
    /// Zip-bomb limit: 256 MiB per entry, decompressed.
    pub max_entry_bytes: u64,
    /// Zip-bomb limit: 1 GiB per pack, decompressed.
    pub max_pack_bytes: u64,
    /// A file larger than this (by the listing) is not read as an archive: `LimitExceeded`.
    /// Default 1 GiB, the per-pack limit.
    pub max_archive_bytes: u64,
    /// Read window: at most this many `ReadFile` requests outstanding (default 64) …
    pub max_pending_reads: u32,
    /// … and at most this many bytes (default 256 MiB). A read is charged `n` for
    /// `Prefix(n)`, else its `len_hint`, else `max_archive_bytes`. Under `Residency::Transient`
    /// a loose file's charge is released when its `Parse` job is fed back, not when the read
    /// is answered; bytes that stay resident (archives, brarchives, `Residency::All`) are
    /// released when answered. A read whose charge exceeds the whole window is issued when no
    /// other read is charged, so an oversized file cannot block the loader.
    pub max_pending_read_bytes: u64,
    /// At most this many jobs issued and not yet fed back (default 4096).
    pub max_pending_jobs: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_entries: 65_536,
            max_entry_bytes: 256 * MIB,
            max_pack_bytes: GIB,
            // An archive may be as large as a whole pack.
            max_archive_bytes: GIB,
            max_pending_reads: 64,
            max_pending_read_bytes: 256 * MIB,
            max_pending_jobs: 4096,
        }
    }
}

/// The game's limit on a JSON document, 128 MiB: a longer one is refused ("Minecraft Document
/// JSON exceeds the maximum size of {} bytes").
pub const MAX_JSON_DOCUMENT_BYTES: usize = 0x0800_0000;

/// From this length on, 96 MiB or three quarters of the limit, a JSON document is approaching
/// the limit: the game warns and the document is read.
pub const JSON_DOCUMENT_WARNING_BYTES: usize = 3 << 25;

/// Where a JSON document's length stands against the game's limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsonDocumentSize {
    /// Below [`JSON_DOCUMENT_WARNING_BYTES`].
    Within,
    /// At or above [`JSON_DOCUMENT_WARNING_BYTES`], at most [`MAX_JSON_DOCUMENT_BYTES`].
    Approaching,
    /// Above [`MAX_JSON_DOCUMENT_BYTES`]: refused.
    TooLarge,
}

/// Classifies a JSON document of `len` bytes.
pub fn json_document_size(len: usize) -> JsonDocumentSize {
    json_document_size_against(len, MAX_JSON_DOCUMENT_BYTES)
}

/// [`json_document_size`] against the limit `max`: refused above it, approaching from three
/// quarters of it (the game's `size >> 25 >= 3` against 2²⁷).
pub fn json_document_size_against(len: usize, max: usize) -> JsonDocumentSize {
    if len > max {
        JsonDocumentSize::TooLarge
    } else if len >= max / 4 * 3 {
        JsonDocumentSize::Approaching
    } else {
        JsonDocumentSize::Within
    }
}
