//! Marketplace decryption with caller-supplied keys: `contents.json` behind its header, and each
//! asset with its own key. Finding a key is the caller's business (`OpenOptions::keys`,
//! `Output::NeedKey`); these functions only apply one.

use aes::Aes256;
use bytes::Bytes;
use cfb8::cipher::{AsyncStreamCipher, KeyIvInit, StreamCipher};

use super::contents::{AssetCipher, ContentsJson};
use super::magic::{ENCRYPTED_HEADER_LEN, FileType, detect};
use super::{ContentKey, StorageError};

/// Decrypts `contents.json` with the pack key and reads the table: the ciphertext starts after
/// the 0x100-byte header, the cipher is AES-256-CFB8 with the key's 32 bytes and an IV of its
/// first 16.
///
/// `Decrypt` when the bytes carry no encryption header or the plaintext is not a contents
/// table (a wrong key decrypts to noise); `MissingContentKey` when the header is there and no
/// key was given.
pub(crate) fn decrypt_contents(
    bytes: &[u8],
    key: Option<&ContentKey>,
) -> Result<ContentsJson, StorageError> {
    let plain = decrypt_behind_header(bytes, key)?;
    ContentsJson::parse(&plain).map_err(|_| StorageError::Decrypt)
}

/// Decrypts a whole file behind the encryption header (a whole-file encrypted zip) with the
/// pack key, the same way as `contents.json`; `Decrypt` unless the plaintext is a zip.
pub(crate) fn decrypt_archive(
    bytes: &[u8],
    key: Option<&ContentKey>,
) -> Result<Bytes, StorageError> {
    let plain = decrypt_behind_header(bytes, key)?;
    if detect(&plain) != FileType::Zip {
        return Err(StorageError::Decrypt);
    }
    Ok(Bytes::from(plain))
}

/// The plaintext after the 0x100-byte header: `Decrypt` without the header,
/// `MissingContentKey` without a key.
fn decrypt_behind_header(bytes: &[u8], key: Option<&ContentKey>) -> Result<Vec<u8>, StorageError> {
    let FileType::Encrypted(_) = detect(bytes) else {
        return Err(StorageError::Decrypt);
    };
    let key = key.ok_or(StorageError::MissingContentKey)?;
    let mut plain = bytes[ENCRYPTED_HEADER_LEN..].to_vec();
    cfb8_decrypt(key.bytes(), &iv(key), &mut plain);
    Ok(plain)
}

/// Decrypts one asset with its own cipher from the contents table. Assets carry no header: the
/// ciphertext starts at offset 0. The key's first 16 bytes are the IV (CFB8) or the initial
/// counter block (CTR).
pub(crate) fn decrypt_asset(bytes: &[u8], cipher: &AssetCipher) -> Result<Bytes, StorageError> {
    let mut plain = bytes.to_vec();
    match cipher {
        AssetCipher::Cfb8(key) => cfb8_decrypt(key.bytes(), &iv(key), &mut plain),
        // The game does not document the counter layout of its counter mode; a 128-bit
        // big-endian counter starting at the IV is the common one, and a pack encrypted that
        // way decrypts.
        AssetCipher::Ctr(key) => ctr_decrypt(key.bytes(), &iv(key), &mut plain),
        AssetCipher::Unusable => return Err(StorageError::Decrypt),
    }
    Ok(Bytes::from(plain))
}

/// The IV of a key: its first 16 bytes.
fn iv(key: &ContentKey) -> [u8; 16] {
    let mut iv = [0; 16];
    iv.copy_from_slice(&key.bytes()[..16]);
    iv
}

/// AES-256-CFB8 decryption in place (8-bit feedback: the game's `Cfb`, not `Cfb128`).
pub(crate) fn cfb8_decrypt(key: &[u8; 32], iv: &[u8; 16], data: &mut [u8]) {
    cfb8::Decryptor::<Aes256>::new(key.into(), iv.into()).decrypt(data);
}

/// AES-256-CTR decryption in place, with a 128-bit big-endian counter starting at `counter`.
pub(crate) fn ctr_decrypt(key: &[u8; 32], counter: &[u8; 16], data: &mut [u8]) {
    ctr::Ctr128BE::<Aes256>::new(key.into(), counter.into()).apply_keystream(data);
}
