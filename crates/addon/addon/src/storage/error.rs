//! Why a file could not be read from a pack.

use bedrock_addon_core::pack_identity::PathRejected;

/// Why a file could not be read from a pack, or a container could not be decoded.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum StorageError {
    /// The zip archive is malformed or uses a feature the reader does not support.
    #[error("the zip archive is malformed or unsupported")]
    Zip {
        /// What is wrong with it.
        #[source]
        fault: ZipFault,
    },
    /// A path inside the container breaks the pack-path rules (a `.` or `..` segment, a
    /// control character, too many segments, too long).
    // The reason is core's `PathRejected` enum; there is no second rule type here.
    #[error("{path}: path rejected")]
    PathRejected {
        /// The path as written in the container.
        path: Box<str>,
        /// The rule it broke.
        #[source]
        reason: PathRejected,
    },
    /// A zip-bomb limit was exceeded (`Limits`).
    #[error("a container limit was exceeded")]
    LimitExceeded {
        /// The limit.
        #[source]
        limit: ZipLimit,
    },
    /// An encrypted entry and no content key for the pack.
    #[error("no content key for this pack")]
    MissingContentKey,
    /// Decryption failed: the key is wrong or the decrypted `contents.json` is not a contents
    /// table.
    #[error("decryption failed")]
    Decrypt,
    /// No such file in the pack.
    #[error("no such file in the pack")]
    NotFound,
    /// The file is indexed but its bytes were not loaded.
    #[error("the file is indexed but not resident")]
    NotResident,
    /// A `.brarchive` file is malformed: wrong magic or version, or an entry outside the file.
    #[error("malformed brarchive")]
    Brarchive,
    /// A `\x7FMCB` binary document could not be turned into JSON text.
    #[error("the binary document could not be turned into JSON text")]
    Mcb {
        /// What stopped it.
        #[source]
        fault: McbFault,
    },
}

/// One of the zip-bomb and archive limits of `Limits`; exceeding any makes the container
/// unsupported.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum ZipLimit {
    /// More entries than `Limits::max_entries` (65 536).
    #[error("too many entries")]
    Entries,
    /// An entry larger than `Limits::max_entry_bytes` (256 MiB) decompressed.
    #[error("entry too large")]
    EntryBytes,
    /// More than `Limits::max_pack_bytes` (1 GiB) decompressed in one archive.
    #[error("archive too large decompressed")]
    PackBytes,
    /// An archive file larger than `Limits::max_archive_bytes`.
    #[error("archive file too large")]
    ArchiveBytes,
    /// An entry name longer than 255 bytes.
    #[error("entry name longer than 255 bytes")]
    EntryName,
    /// Archives or pack folders nested deeper than 4 inside a composite
    /// (`folder_structure_too_deep`).
    #[error("archives or pack folders nested deeper than 4")]
    Nesting,
}

/// What is wrong with a zip archive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum ZipFault {
    /// Not a zip archive, truncated, or internally inconsistent (no end-of-central-directory
    /// record, an entry that inflates past its declared size, a checksum mismatch).
    #[error("malformed")]
    Malformed,
    /// A zip feature the reader does not implement: a compression method other than stored
    /// and deflate, zip-level encryption, a multi-disk archive.
    #[error("unsupported")]
    Unsupported,
}

/// What stopped a `\x7FMCB` binary document from becoming JSON text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum McbFault {
    /// The header (magic, format version, payload key) is cut short or malformed.
    #[error("malformed header")]
    Header,
    /// The body is not one of the layouts the reader knows, or is cut short or malformed (a
    /// body has no type tags, so the two are not told apart): such a document is an
    /// unsupported file format.
    #[error("body layout not supported")]
    UnsupportedBody,
    /// A float that is NaN or infinite, which JSON cannot write.
    #[error("non-finite float")]
    NonFiniteFloat,
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn wrapped_faults_are_sources_not_message_text() {
        let zip = StorageError::Zip {
            fault: ZipFault::Malformed,
        };
        assert!(zip.source().is_some(), "the zip fault is the source");
        assert!(!zip.to_string().contains("malformed zip"), "{zip}");
        let limit = StorageError::LimitExceeded {
            limit: ZipLimit::Entries,
        };
        assert!(limit.source().is_some());
        assert!(!limit.to_string().contains("too many entries"), "{limit}");
        let mcb = StorageError::Mcb {
            fault: McbFault::Header,
        };
        assert!(mcb.source().is_some());
        assert_eq!(mcb.clone(), mcb);
    }
}
