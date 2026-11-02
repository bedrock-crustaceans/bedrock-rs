//! Pack paths, storage errors and content keys. The container formats themselves (zip,
//! encryption, brarchive, MCB) are crate-private readers over resident bytes.
//!
//! Every container is told apart by magic bytes, never by extension. Zip archives are decoded
//! in memory, under the zip-bomb limits of `Limits` (65 536 entries, 256 MiB per entry, 1 GiB
//! per archive decompressed) and the pack-path rules, and written with a fixed time so the same
//! files give the same bytes. Marketplace packs keep a plaintext `manifest.json`; their
//! `contents.json` and assets are decrypted with keys the caller supplies ([`ContentKeys`]).
//! A pack's `__brarchive/*.brarchive` files are read before its loose files.
// The crate-private interface, tested beside its code: `magic::detect`, `zip::{ZipIndex, encode}`,
// `contents::{ContentsJson, ContentsTable}`, `crypt::{decrypt_contents, decrypt_archive,
// decrypt_asset}`, `brarchive::{Brarchive, BrarchiveLayer, archive_dir}`,
// `mcb::{McbHeader, to_json_text}`.

pub(crate) mod brarchive;
// The contents table decrypts assets only with `encrypted`.
#[cfg_attr(not(feature = "encrypted"), allow(dead_code))]
pub(crate) mod contents;
#[cfg(feature = "encrypted")]
pub(crate) mod crypt;
mod error;
mod key;
// `ENCRYPTED_HEADER_LEN` is read by `crypt` only, which needs the
// `encrypted` feature.
#[cfg_attr(not(feature = "encrypted"), allow(dead_code))]
pub(crate) mod magic;
pub(crate) mod mcb;
#[cfg(feature = "zip")]
pub(crate) mod zip;

pub use bedrock_addon_core::pack_identity::{PackPath, PathRejected};
pub use error::{McbFault, StorageError, ZipFault, ZipLimit};
pub use key::{ContentIdentity, ContentKey, ContentKeys};

#[cfg(test)]
mod tests;
