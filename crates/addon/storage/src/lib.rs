//! The containers of a Minecraft Bedrock pack, as plain functions over resident bytes: zip
//! archives, marketplace encryption, `__brarchive` archives and `\x7FMCB` documents. Sans-IO:
//! the caller reads the files; `bedrock_addon` re-exports the public set as
//! `bedrock_addon::storage` and is the one user of the container modules.
//!
//! Every container is told apart by magic bytes, never by extension. Zip archives are decoded
//! in memory, under the zip-bomb limits of `Limits` (65 536 entries, 256 MiB per entry, 1 GiB
//! per archive decompressed) and the pack-path rules, and written with a fixed time so the same
//! files give the same bytes. Marketplace packs keep a plaintext `manifest.json`; their
//! `contents.json` and assets are decrypted with keys the caller supplies ([`ContentKeys`]).
//! A pack's `__brarchive/*.brarchive` files are read before its loose files.
// The container modules are `pub` for `bedrock_addon` alone and hidden from the docs: they are
// the loader's and the writer's internal surface, not an API of their own. The interface, tested
// beside its code: `magic::detect`, `zip::{ZipIndex, encode}`, `contents::{ContentsJson,
// ContentsTable}`, `crypt::{decrypt_contents, decrypt_archive, decrypt_asset}`,
// `brarchive::{Brarchive, BrarchiveLayer, archive_dir}`, `mcb::{McbHeader, to_json_text}`.

// Internal: the brarchive reader the loader and the writer use.
#[doc(hidden)]
pub mod brarchive;
// Internal: the `contents.json` of a marketplace pack, for the loader.
#[doc(hidden)]
pub mod contents;
// Internal: marketplace decryption, for the loader and `PackStack::open`.
#[cfg(feature = "encrypted")]
#[doc(hidden)]
pub mod crypt;
mod error;
mod key;
// Internal: the magic-byte sniffing every container read starts with.
#[doc(hidden)]
pub mod magic;
// Internal: the MCB decoder the stack and the writer turn binary documents into JSON text with.
#[doc(hidden)]
pub mod mcb;
// Internal: the in-memory zip reader and writer of the loader and the writer.
#[cfg(feature = "zip")]
#[doc(hidden)]
pub mod zip;

pub use bedrock_addon_core::pack_identity::{PackPath, PathRejected};
pub use error::{McbFault, StorageError, ZipFault, ZipLimit};
pub use key::{ContentIdentity, ContentKey, ContentKeys};

#[cfg(test)]
mod tests;
