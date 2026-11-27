//! Content key lookup: the provider trait a `Driver` asks when a loader needs a key, and the
//! `<archive>.key` sidecar convention.

use bedrock_addon::load::KeyRequest;
use bedrock_addon::storage::{ContentKey, ContentKeys};

/// Supplies content keys when a loader asks (`Output::NeedKey`).
pub trait ContentKeyProvider: Send + Sync {
    /// The key for the pack `request` names, if known.
    fn key_for(&self, request: &KeyRequest) -> Option<ContentKey>;
}

/// A plain map answers by identity.
impl ContentKeyProvider for ContentKeys {
    fn key_for(&self, request: &KeyRequest) -> Option<ContentKey> {
        self.get(&request.identity).cloned()
    }
}

/// Reads `<archive file name>.key` (32 ASCII characters) next to the pack's file, given
/// the host path the driver resolved from `KeyRequest::location`. This crate's own
/// convention, not vanilla's. Used by `Driver` only when `Options::sidecar_keys` is set; a
/// custom loop calls it with `Roots::host_path`.
///
/// The sidecar of `packs/ruby.mcpack` is `packs/ruby.mcpack.key`, and of a pack folder
/// `packs/ruby` it is `packs/ruby.key`. ASCII whitespace around the 32 characters (a final
/// newline) is ignored; anything else (a missing or unreadable file, another length, a
/// character that is not printable ASCII) is no key.
#[cfg(feature = "encrypted")]
#[cfg_attr(docsrs, doc(cfg(feature = "encrypted")))]
pub fn read_sidecar_key(pack_file: impl AsRef<std::path::Path>) -> Option<ContentKey> {
    use std::io::Read;

    // A key is exactly 32 printable ASCII bytes; ASCII whitespace around it (a final newline) is
    // allowed and trimmed.
    const KEY_LEN: usize = 32;
    // Enough for the key and generous surrounding whitespace; a larger file is not a key.
    const READ_LIMIT: u64 = 256;

    let pack_file = pack_file.as_ref();
    let mut name = pack_file.file_name()?.to_os_string();
    name.push(".key");
    let sidecar = pack_file.with_file_name(name);
    if !std::fs::metadata(&sidecar).ok()?.is_file() {
        return None;
    }
    let mut text = Vec::with_capacity(KEY_LEN + 2);
    std::fs::File::open(&sidecar)
        .ok()?
        .take(READ_LIMIT)
        .read_to_end(&mut text)
        .ok()?;
    let key = text.trim_ascii();
    if key.len() != KEY_LEN || !key.iter().all(u8::is_ascii_graphic) {
        return None;
    }
    ContentKey::from_slice(key)
}
