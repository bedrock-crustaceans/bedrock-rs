//! What a file is, from its first bytes: the game's file-type test plus the in-pack formats.

use super::key::ContentIdentity;

/// How many leading bytes [`detect`] looks at; a loader requests `ReadRange::Prefix(256)`.
pub const PREFIX_LEN: usize = 256;

/// The kind of a file, by magic bytes, never by extension.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileType {
    /// A zip archive (`.mcpack`, `.zip`, `.mcaddon`, `.mctemplate`, `.mcworld`).
    Zip,
    /// A file behind the marketplace encryption header: `contents.json`, or a whole-file
    /// encrypted zip.
    Encrypted(ContentIdentity),
    /// A `.brarchive`.
    Brarchive,
    /// A `\x7FMCB` binary document.
    Mcb,
    /// Anything else (JSON text, an image, an asset the encryption layer still hides). As the
    /// type of a pack file, the game's `Unknown`: an unsupported format.
    Plain,
}

/// `PK\x03\x04`, the zip local-file-header signature.
const ZIP_MAGIC: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];
/// The brarchive header magic, u64 `0x267052A0B125277D` little-endian.
const BRARCHIVE_MAGIC: [u8; 8] = 0x2670_52A0_B125_277D_u64.to_le_bytes();
/// `\x7FMCB`, i32 `0x42434D7F` little-endian.
pub const MCB_MAGIC: [u8; 4] = [0x7F, b'M', b'C', b'B'];
/// u32 `0x9BCFB9FC` at offset 4 of the encryption header.
const ENCRYPTED_MAGIC: [u8; 4] = 0x9BCF_B9FC_u32.to_le_bytes();
/// The encryption header's size: the ciphertext of `contents.json` starts here.
pub const ENCRYPTED_HEADER_LEN: usize = 0x100;
/// The longest identity the header may hold.
const MAX_IDENTITY_LEN: usize = 0xEE;

/// The type of the file whose first bytes are `prefix` (at most [`PREFIX_LEN`] are looked at).
///
/// The game's order: the zip signature first (at any length); a prefix shorter than 256 bytes is
/// unknown; then the encryption header. The brarchive and MCB magics are the crate's own: the
/// game tells those apart where it opens them.
pub fn detect(prefix: &[u8]) -> FileType {
    // The zip local-file-header signature at offset 0.
    if prefix.starts_with(&ZIP_MAGIC) {
        return FileType::Zip;
    }
    if prefix.starts_with(&BRARCHIVE_MAGIC) {
        return FileType::Brarchive;
    }
    if prefix.starts_with(&MCB_MAGIC) {
        return FileType::Mcb;
    }
    match encrypted_identity(prefix) {
        Some(identity) => FileType::Encrypted(identity),
        None => FileType::Plain,
    }
}

/// The `ContentIdentity` of an encryption header: at least 256 bytes, u32@0 == 0, u32@4 ==
/// `0x9BCFB9FC`, `len` = u8@0x10 ≤ 0xEE, and the string at 0x11 NUL-terminated with exactly
/// `len` bytes before the NUL.
fn encrypted_identity(prefix: &[u8]) -> Option<ContentIdentity> {
    let header = prefix.get(..PREFIX_LEN)?;
    if header[0..4] != [0; 4] || header[4..8] != ENCRYPTED_MAGIC {
        return None;
    }
    let len = usize::from(header[0x10]);
    if len > MAX_IDENTITY_LEN {
        return None;
    }
    let text = &header[0x11..];
    let nul = text.iter().position(|&byte| byte == 0)?;
    if nul != len {
        return None;
    }
    // The identity is ASCII in every pack the game ships (a UUID); other bytes are kept lossily.
    Some(ContentIdentity::new(String::from_utf8_lossy(&text[..len])))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(identity: &[u8], len: u8) -> Vec<u8> {
        let mut bytes = vec![0u8; PREFIX_LEN];
        bytes[4..8].copy_from_slice(&[0xFC, 0xB9, 0xCF, 0x9B]);
        bytes[0x10] = len;
        bytes[0x11..0x11 + identity.len()].copy_from_slice(identity);
        bytes
    }

    #[test]
    fn zip_local_header_is_a_zip_at_any_length() {
        assert_eq!(detect(b"PK\x03\x04"), FileType::Zip);
    }

    #[test]
    fn end_of_central_directory_first_is_not_a_zip() {
        // the game tests only the local-header magic at offset 0
        assert_eq!(
            detect(b"PK\x05\x06\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"),
            FileType::Plain
        );
    }

    #[test]
    fn encrypted_header_names_its_identity() {
        assert_eq!(
            detect(&header(b"pack-identity", 13)),
            FileType::Encrypted(ContentIdentity::new("pack-identity"))
        );
    }

    #[test]
    fn encrypted_header_needs_256_bytes() {
        assert_eq!(
            detect(&header(b"pack-identity", 13)[..255]),
            FileType::Plain
        );
    }

    #[test]
    fn encrypted_header_length_must_match_the_string() {
        assert_eq!(detect(&header(b"pack-identity", 12)), FileType::Plain);
        assert_eq!(detect(&header(b"pack-identity", 14)), FileType::Plain);
    }

    #[test]
    fn encrypted_header_length_is_at_most_0xee() {
        let identity = vec![b'a'; 0xEF];
        assert_eq!(detect(&header(&identity, 0xEF)), FileType::Plain);
        let identity = vec![b'a'; 0xEE];
        assert!(matches!(
            detect(&header(&identity, 0xEE)),
            FileType::Encrypted(_)
        ));
    }

    #[test]
    fn encrypted_header_needs_version_zero() {
        let mut bytes = header(b"pack-identity", 13);
        bytes[0] = 1;
        assert_eq!(detect(&bytes), FileType::Plain);
    }

    #[test]
    fn brarchive_magic() {
        let bytes = brarchive::serialize([("a.json", "{}")]).unwrap();
        assert_eq!(detect(&bytes), FileType::Brarchive);
    }

    #[test]
    fn mcb_magic() {
        assert_eq!(detect(b"\x7fMCB\x01\x00\x1a\x00"), FileType::Mcb);
    }

    #[test]
    fn json_text_is_plain() {
        assert_eq!(detect(br#"{"format_version": 2}"#), FileType::Plain);
        assert_eq!(detect(b""), FileType::Plain);
    }
}
