//! The sans-IO loaders: `Pack::from_files` / `from_archive` over the fixtures, the
//! `PackLoader` request sequences, the `RepositoryLoader` and `StackLoader`, driven by the
//! scripted responder of `common/responder.rs` (12 "Loader conformance layer").

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
// The archive and encryption tests compile only with their features.
#![cfg_attr(
    not(all(feature = "zip", feature = "encrypted")),
    allow(unused_imports, dead_code)
)]

#[path = "common/responder.rs"]
mod responder;

#[path = "loader/open.rs"]
mod open;
#[path = "loader/pack_loader.rs"]
mod pack_loader;
#[path = "loader/protocol.rs"]
mod protocol;
#[path = "loader/repository.rs"]
mod repository;
#[path = "loader/stack.rs"]
mod stack;

use std::path::PathBuf;

/// The fixture directory.
fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// A fixture's bytes.
fn fixture(relative: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(relative)).unwrap_or_else(|e| panic!("{relative}: {e}"))
}

/// `Pack::to_archive` over whole-file encrypted archives (the original archive is written;
/// content keys and plaintext never are).
#[cfg(feature = "encrypted")]
mod encrypted_archive {
    use std::io::{Cursor, Write};

    use bedrock_addon::load::{RepositoryLoader, RootId, Source};
    use bedrock_addon::pack::{OpenOptions, Pack, PackOrigin, WriteError};
    use bedrock_addon::storage::{ContentIdentity, ContentKey, ContentKeys};

    use super::fixture;
    use super::responder::{Order, Responder, manifest};

    /// A stored zip of `members` (name, bytes).
    fn zip(members: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, bytes) in members {
            writer
                .start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    /// The key `storage/encrypted_archive/whole.mcpack` was built with.
    const PACK_KEY: &[u8; 32] = b"pack-key-for-the-synthetic-pack!";

    /// AES-256-CFB8 encryption with the key's first 16 bytes as IV (the cipher of 03).
    fn cfb8_encrypt(key: &[u8; 32], plain: &[u8]) -> Vec<u8> {
        use aes::cipher::{BlockEncrypt, KeyInit};
        let cipher = aes::Aes256::new(key.into());
        let mut register: [u8; 16] = key[..16].try_into().unwrap();
        plain
            .iter()
            .map(|byte| {
                let mut block = aes::Block::from(register);
                cipher.encrypt_block(&mut block);
                let out = byte ^ block[0];
                register.copy_within(1.., 0);
                register[15] = out;
                out
            })
            .collect()
    }

    /// `zip` behind the encryption header of `whole.mcpack`, and options holding its key.
    fn encrypted(zip: &[u8]) -> (Vec<u8>, OpenOptions) {
        let header =
            &fixture("../../../storage/tests/fixtures/encrypted_archive/whole.mcpack")[..0x100];
        let mut bytes = header.to_vec();
        bytes.extend(cfb8_encrypt(PACK_KEY, zip));
        let keys: ContentKeys = [(
            ContentIdentity::new("00000000-0000-4000-8000-0000000a2c41"),
            ContentKey::new(*PACK_KEY),
        )]
        .into_iter()
        .collect();
        let options = OpenOptions {
            keys,
            ..OpenOptions::default()
        };
        (bytes, options)
    }

    #[test]
    fn a_whole_file_encrypted_archive_with_its_pack_in_a_folder_zips_back_to_its_ciphertext() {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file(
                "ruby/manifest.json",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
        writer.write_all(manifest(6, "data").as_bytes()).unwrap();
        let zip = writer.finish().unwrap().into_inner();
        let (bytes, options) = encrypted(&zip);
        let pack = Pack::from_archive(bytes.clone(), &options).unwrap();
        assert_eq!(pack.location().path.as_str(), "ruby");
        let served = pack.to_archive().unwrap();
        assert!(served == bytes, "the original, never the plaintext");
    }

    #[test]
    fn the_packs_of_a_whole_file_encrypted_composite_are_not_zipped() {
        let nested = zip(&[("manifest.json", manifest(8, "resources").as_bytes())]);
        let outer = zip(&[
            ("bp/manifest.json", manifest(7, "data").as_bytes()),
            ("rp.mcpack", &nested),
        ]);
        let (bytes, options) = encrypted(&outer);
        let mut responder = Responder::new(Order::Issue);
        responder.root_file(4, bytes);
        let source = Source::Archive {
            root: RootId(4),
            origin: PackOrigin::World,
        };
        let mut loader = RepositoryLoader::new(vec![source], options);
        let repository = responder.drive(&mut loader);
        assert_eq!(repository.packs().count(), 2, "{:?}", repository.invalid());
        for pack in repository.packs() {
            let served = pack.to_archive();
            assert!(
                matches!(served, Err(WriteError::Encrypted)),
                "{}: {served:?}",
                pack.location()
            );
        }
    }
}
