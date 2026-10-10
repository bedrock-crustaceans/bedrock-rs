use crate::error::EncryptionError;
use aes::Aes256;
use ctr::cipher::{StreamCipher, StreamCipherSeek};
use ctr::{Ctr128BE, cipher::KeyIvInit};
use p384::{PublicKey, SecretKey};
use sha2::{Digest, Sha256};
use std::fmt;

pub struct Encryption {
    encrypt_counter: u64,
    encrypt_cipher: Ctr128BE<Aes256>,
    decrypt_counter: u64,
    decrypt_cipher: Ctr128BE<Aes256>,
    key: [u8; 32],
}

impl fmt::Debug for Encryption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Encryption")
            .field("encrypt_counter", &self.encrypt_counter)
            .field("decrypt_counter", &self.decrypt_counter)
            .finish_non_exhaustive()
    }
}

/// Serializable [`Encryption`] state, from which the ciphers are rebuilt using the key
/// and each direction's keystream position.
#[derive(Clone, Debug, facet::Facet)]
pub struct EncryptionSnapshot {
    key: [u8; 32],
    encrypt_counter: u64,
    encrypt_pos: u64,
    decrypt_counter: u64,
    decrypt_pos: u64,
}

fn iv_from_key(key: &[u8; 32]) -> [u8; 16] {
    let mut iv = [0u8; 16];
    iv[..12].copy_from_slice(&key[..12]);
    iv[15] = 2;
    iv
}

impl Encryption {
    pub fn new(secret: &SecretKey, public: &PublicKey, salt: &[u8]) -> Self {
        let shared = p384::ecdh::diffie_hellman(secret.to_nonzero_scalar(), public.as_affine());

        let shared_bytes = shared.raw_secret_bytes();

        let mut hasher = Sha256::new();
        hasher.update(salt);
        hasher.update(shared_bytes);
        let key = hasher.finalize();
        let key: [u8; 32] = key.into();

        let iv = iv_from_key(&key);

        let encrypt_cipher = Ctr128BE::<Aes256>::new(&key.into(), (&iv).into());
        let decrypt_cipher = Ctr128BE::<Aes256>::new(&key.into(), (&iv).into());

        Self {
            encrypt_counter: 0,
            encrypt_cipher,
            decrypt_counter: 0,
            decrypt_cipher,
            key,
        }
    }

    pub fn snapshot(&self) -> EncryptionSnapshot {
        EncryptionSnapshot {
            key: self.key,
            encrypt_counter: self.encrypt_counter,
            encrypt_pos: self.encrypt_cipher.current_pos(),
            decrypt_counter: self.decrypt_counter,
            decrypt_pos: self.decrypt_cipher.current_pos(),
        }
    }

    pub fn restore(state: EncryptionSnapshot) -> Self {
        let iv = iv_from_key(&state.key);

        let mut encrypt_cipher = Ctr128BE::<Aes256>::new(&state.key.into(), (&iv).into());
        encrypt_cipher.seek(state.encrypt_pos);

        let mut decrypt_cipher = Ctr128BE::<Aes256>::new(&state.key.into(), (&iv).into());
        decrypt_cipher.seek(state.decrypt_pos);

        Self {
            encrypt_counter: state.encrypt_counter,
            encrypt_cipher,
            decrypt_counter: state.decrypt_counter,
            decrypt_cipher,
            key: state.key,
        }
    }

    pub fn encrypt(&mut self, buf: Vec<u8>) -> Result<Vec<u8>, EncryptionError> {
        let trailer = self.trailer(&buf, self.encrypt_counter);

        let mut out = Vec::<u8>::with_capacity(buf.len() + trailer.len());
        out.extend_from_slice(&buf);
        out.extend_from_slice(&trailer);

        self.encrypt_cipher.apply_keystream(&mut out);

        self.encrypt_counter += 1;

        Ok(out)
    }

    pub fn decrypt(&mut self, buf: Vec<u8>) -> Result<Vec<u8>, EncryptionError> {
        if buf.len() <= 8 {
            return Err(EncryptionError::InvalidLength(buf.len()));
        }

        let mut out = buf;
        self.decrypt_cipher.apply_keystream(&mut out);

        let trailer = &out[out.len() - 8..];
        let expected_trailer = self.trailer(&out[..out.len() - 8], self.decrypt_counter);
        if trailer != expected_trailer {
            return Err(EncryptionError::InvalidTrailer);
        }

        self.decrypt_counter += 1;

        out.truncate(out.len() - 8);
        Ok(out)
    }

    pub fn trailer(&self, buf: &[u8], counter: u64) -> [u8; 8] {
        let mut hasher = Sha256::new();
        hasher.update(counter.to_le_bytes());
        hasher.update(buf);
        hasher.update(self.key);
        let hash = hasher.finalize();

        let mut trailer = [0u8; 8];
        trailer.copy_from_slice(&hash[..8]);
        trailer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use p384::SecretKey;

    fn matched_pair() -> (Encryption, Encryption) {
        let token = [7u8; 16];
        let a_secret = SecretKey::from_slice(&[0x11u8; 48]).unwrap();
        let b_secret = SecretKey::from_slice(&[0x22u8; 48]).unwrap();

        let a = Encryption::new(&a_secret, &b_secret.public_key(), &token);
        let b = Encryption::new(&b_secret, &a_secret.public_key(), &token);
        (a, b)
    }

    #[test]
    fn restored_snapshot_continues_keystream() {
        let (mut a, mut b) = matched_pair();

        for i in 0..3 {
            let msg = format!("packet {i}").into_bytes();
            let ct = a.encrypt(msg.clone()).unwrap();
            assert_eq!(b.decrypt(ct).unwrap(), msg);
        }

        let json = facet_json::to_string(&a.snapshot()).unwrap();
        let mut a_resumed = Encryption::restore(facet_json::from_str(&json).unwrap());

        let msg = b"after restore".to_vec();
        let ct = a_resumed.encrypt(msg.clone()).unwrap();
        assert_eq!(b.decrypt(ct).unwrap(), msg);

        let msg = b"reply".to_vec();
        let ct = b.encrypt(msg.clone()).unwrap();
        assert_eq!(a_resumed.decrypt(ct).unwrap(), msg);
    }

    #[test]
    fn ciphertext_is_unchanged_for_a_fixed_key_pair() {
        let (mut a, _) = matched_pair();

        let first = a.encrypt(b"first packet".to_vec()).unwrap();
        let second = a.encrypt(b"second packet".to_vec()).unwrap();

        assert_eq!(
            first,
            [
                0xd5, 0x01, 0xf1, 0x63, 0x22, 0x83, 0xc8, 0x3b, 0x24, 0xd3, 0x04, 0x48, 0xe2, 0x94,
                0x91, 0xd8, 0x8c, 0xf4, 0xf3, 0x88,
            ]
        );
        assert_eq!(
            second,
            [
                0x54, 0xb9, 0x95, 0xad, 0xaa, 0x41, 0x92, 0x50, 0x9a, 0x24, 0x14, 0xaf, 0x2b, 0xb8,
                0xed, 0x7d, 0x26, 0x99, 0xbc, 0x21, 0x21,
            ]
        );
    }
}
