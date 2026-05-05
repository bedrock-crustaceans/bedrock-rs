use crate::error::EncryptionError;
use aes::Aes256;
use ctr::cipher::StreamCipher;
use ctr::{Ctr128BE, cipher::KeyIvInit};
use p384::{PublicKey, SecretKey};
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct Encryption {
    encrypt_counter: u64,
    encrypt_cipher: Ctr128BE<Aes256>,
    decrypt_counter: u64,
    decrypt_cipher: Ctr128BE<Aes256>,
    key: [u8; 32],
}

impl Encryption {
    pub fn new(secret: &SecretKey, public: &PublicKey, token: &[u8; 16]) -> Self {
        let shared = secret.diffie_hellman(public);

        let shared_bytes = shared.raw_secret_bytes();

        let mut hasher = Sha256::new();
        hasher.update(token);
        hasher.update(shared_bytes);
        let key = hasher.finalize();

        let mut iv = [0u8; 16];
        iv[..12].copy_from_slice(&key[..12]);
        iv[15] = 2;

        let encrypt_cipher = Ctr128BE::<Aes256>::new(&key, (&iv).into());
        let decrypt_cipher = Ctr128BE::<Aes256>::new(&key, (&iv).into());

        Self {
            encrypt_counter: 0,
            encrypt_cipher,
            decrypt_counter: 0,
            decrypt_cipher,
            key: key.into(),
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
