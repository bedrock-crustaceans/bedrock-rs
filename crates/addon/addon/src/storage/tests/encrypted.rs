//! Marketplace encryption: the vanilla test pack (no key ships with it) and a synthetic pack
//! encrypted by `openssl enc` (an independent implementation) with known keys.

use super::fixture;
use crate::storage::contents::{AssetCipher, ContentsTable};
use crate::storage::crypt::{cfb8_decrypt, ctr_decrypt, decrypt_asset, decrypt_contents};
use crate::storage::magic::{FileType, detect};
use crate::storage::{ContentIdentity, ContentKey, PackPath, StorageError};

/// The keys `encrypted_synthetic/` was built with.
const PACK_KEY: &[u8; 32] = b"pack-key-for-the-synthetic-pack!";
const RUBY_KEY: &[u8; 32] = b"asset-key-ruby-json-cfb8-0123456";
const TEXT_KEY: &[u8; 32] = b"asset-key-ruby-text-ctr-01234567";

fn synthetic_table() -> ContentsTable {
    let contents = decrypt_contents(
        &fixture("encrypted_synthetic/contents.json"),
        Some(&ContentKey::new(*PACK_KEY)),
    )
    .unwrap();
    ContentsTable::from_json(&contents)
}

/// A whole archive behind the encryption header (`contents.json` and a whole-file
/// encrypted zip carry the 0x100-byte header), built with `openssl enc` from
/// `synthetic/valid_reference.zip` and the synthetic pack key.
mod whole_archive {
    use super::*;
    use crate::storage::crypt::decrypt_archive;

    #[test]
    fn decrypts_to_the_zip_it_hides() {
        let bytes = fixture("encrypted_archive/whole.mcpack");
        assert_eq!(
            detect(&bytes),
            FileType::Encrypted(ContentIdentity::new("00000000-0000-4000-8000-0000000a2c41"))
        );
        let zip = decrypt_archive(&bytes, Some(&ContentKey::new(*PACK_KEY))).unwrap();
        assert_eq!(zip, fixture("synthetic/valid_reference.zip"));
    }

    #[test]
    fn without_a_key_it_is_missing_its_key() {
        assert!(matches!(
            decrypt_archive(&fixture("encrypted_archive/whole.mcpack"), None),
            Err(StorageError::MissingContentKey)
        ));
    }

    #[test]
    fn a_wrong_key_does_not_give_a_zip() {
        assert!(matches!(
            decrypt_archive(
                &fixture("encrypted_archive/whole.mcpack"),
                Some(&ContentKey::new(*RUBY_KEY))
            ),
            Err(StorageError::Decrypt)
        ));
    }
}

mod vanilla_test_pack {
    use super::*;

    #[test]
    fn header_names_the_identity() {
        assert_eq!(
            detect(&fixture("encrypted/testPack/contents.json")),
            FileType::Encrypted(ContentIdentity::new("325FCAEA-29F0-4D51-AB14-A0BE8C66DC97"))
        );
    }

    #[test]
    fn without_a_key_contents_json_is_missing_its_key() {
        assert!(matches!(
            decrypt_contents(&fixture("encrypted/testPack/contents.json"), None),
            Err(StorageError::MissingContentKey)
        ));
    }

    #[test]
    fn a_wrong_key_fails_to_decrypt() {
        let wrong = ContentKey::new([b'k'; 32]);
        assert!(matches!(
            decrypt_contents(&fixture("encrypted/testPack/contents.json"), Some(&wrong)),
            Err(StorageError::Decrypt)
        ));
    }
}

mod synthetic_pack {
    use super::*;

    #[test]
    fn contents_json_decrypts_with_the_pack_key() {
        let contents = decrypt_contents(
            &fixture("encrypted_synthetic/contents.json"),
            Some(&ContentKey::new(*PACK_KEY)),
        )
        .unwrap();
        assert_eq!(contents.version, Some(1));
        let paths: Vec<&str> = contents
            .content
            .iter()
            .map(|entry| entry.path.as_str())
            .collect();
        assert_eq!(
            paths,
            ["manifest.json", "blocks/ruby.json", "texts/ruby.lang"]
        );
    }

    #[test]
    fn a_wrong_pack_key_fails_to_decrypt() {
        assert!(matches!(
            decrypt_contents(
                &fixture("encrypted_synthetic/contents.json"),
                Some(&ContentKey::new(*RUBY_KEY))
            ),
            Err(StorageError::Decrypt)
        ));
    }

    #[test]
    fn a_file_without_the_header_is_not_encrypted_contents() {
        assert!(matches!(
            decrypt_contents(
                &fixture("encrypted_synthetic/manifest.json"),
                Some(&ContentKey::new(*PACK_KEY))
            ),
            Err(StorageError::Decrypt)
        ));
    }

    #[test]
    fn cfb8_asset_decrypts_with_its_own_key() {
        let table = synthetic_table();
        let cipher = table
            .get(&PackPath::from_static("blocks/ruby.json"))
            .unwrap();
        assert_eq!(cipher, &AssetCipher::Cfb8(ContentKey::new(*RUBY_KEY)));
        let plain =
            decrypt_asset(&fixture("encrypted_synthetic/blocks/ruby.json"), cipher).unwrap();
        assert_eq!(
            plain,
            &br#"{"format_version":"1.21.0","minecraft:block":{"description":{"identifier":"demo:ruby"}}}
"#[..]
        );
    }

    #[test]
    fn ctr_asset_decrypts_with_its_own_key() {
        let table = synthetic_table();
        let cipher = table
            .get(&PackPath::from_static("texts/ruby.lang"))
            .unwrap();
        assert_eq!(cipher, &AssetCipher::Ctr(ContentKey::new(*TEXT_KEY)));
        let plain = decrypt_asset(&fixture("encrypted_synthetic/texts/ruby.lang"), cipher).unwrap();
        assert_eq!(plain, &b"tile.demo:ruby.name=Ruby Block\n"[..]);
    }

    #[test]
    fn an_unusable_asset_key_fails_to_decrypt() {
        assert!(matches!(
            decrypt_asset(b"bytes", &AssetCipher::Unusable),
            Err(StorageError::Decrypt)
        ));
    }
}

/// NIST SP 800-38A known answers: CFB8-AES256.Encrypt (F.3.17) and CTR-AES256.Encrypt (F.5.5),
/// read backwards. They pin 8-bit feedback (vanilla's `Cfb`, not `Cfb128`) and a 128-bit
/// big-endian counter.
mod known_answers {
    use super::*;

    const KEY: [u8; 32] = [
        0x60, 0x3d, 0xeb, 0x10, 0x15, 0xca, 0x71, 0xbe, 0x2b, 0x73, 0xae, 0xf0, 0x85, 0x7d, 0x77,
        0x81, 0x1f, 0x35, 0x2c, 0x07, 0x3b, 0x61, 0x08, 0xd7, 0x2d, 0x98, 0x10, 0xa3, 0x09, 0x14,
        0xdf, 0xf4,
    ];

    #[test]
    fn cfb8_aes256() {
        let iv: [u8; 16] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        let mut data = vec![
            0xdc, 0x1f, 0x1a, 0x85, 0x20, 0xa6, 0x4d, 0xb5, 0x5f, 0xcc, 0x8a, 0xc5, 0x54, 0x84,
            0x4e, 0x88, 0x97, 0x00,
        ];
        cfb8_decrypt(&KEY, &iv, &mut data);
        assert_eq!(
            data,
            [
                0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93,
                0x17, 0x2a, 0xae, 0x2d
            ]
        );
    }

    #[test]
    fn ctr_aes256() {
        let counter: [u8; 16] = [
            0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
            0xfe, 0xff,
        ];
        let mut data = vec![
            0x60, 0x1e, 0xc3, 0x13, 0x77, 0x57, 0x89, 0xa5, 0xb7, 0xa7, 0xf5, 0x04, 0xbb, 0xf3,
            0xd2, 0x28,
        ];
        ctr_decrypt(&KEY, &counter, &mut data);
        assert_eq!(
            data,
            [
                0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93,
                0x17, 0x2a
            ]
        );
    }
}
