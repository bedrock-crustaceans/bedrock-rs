//! Pack paths, storage errors and content keys. The container formats themselves (zip,
//! encryption, brarchive, MCB) are readers over resident bytes in `bedrock_addon_storage`;
//! the loader and the writer reach them through this module.
//!
//! Every container is told apart by magic bytes, never by extension. Zip archives are decoded
//! in memory, under the zip-bomb limits of `Limits` (65 536 entries, 256 MiB per entry, 1 GiB
//! per archive decompressed) and the pack-path rules, and written with a fixed time so the same
//! files give the same bytes. Marketplace packs keep a plaintext `manifest.json`; their
//! `contents.json` and assets are decrypted with keys the caller supplies ([`ContentKeys`]).
//! A pack's `__brarchive/*.brarchive` files are read before its loose files.

pub use bedrock_addon_storage::{
    ContentIdentity, ContentKey, ContentKeys, McbFault, PackPath, PathRejected, StorageError,
    ZipFault, ZipLimit,
};

// The container modules, for the loader and the writer; not part of this crate's API.
#[cfg(feature = "encrypted")]
pub(crate) use bedrock_addon_storage::crypt;
#[cfg(feature = "zip")]
pub(crate) use bedrock_addon_storage::zip;
pub(crate) use bedrock_addon_storage::{brarchive, contents, magic, mcb};

#[cfg(test)]
mod tests;
