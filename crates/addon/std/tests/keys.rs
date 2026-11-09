//! `keys`: the provider a `Driver` asks for content keys, and the `<file>.key` sidecar.

use bedrock_addon::load::{KeyRequest, Place, RootId};
use bedrock_addon::pack::{PackIdVersion, PackType};
use bedrock_addon::storage::{ContentIdentity, ContentKey, ContentKeys};
use bedrock_addon::version::SemVersion;
use bedrock_addon_std::keys::ContentKeyProvider;

const KEY: &[u8; 32] = b"0123456789abcdefghijklmnopqrstuv";

fn key_request(identity: &str) -> KeyRequest {
    KeyRequest {
        pack: PackIdVersion::new(
            uuid::Uuid::from_u128(0x5eed),
            SemVersion::new(1, 0, 0),
            PackType::Resource,
        ),
        identity: ContentIdentity::new(identity),
        location: Place::root(RootId(0)),
    }
}

#[test]
fn content_keys_answer_by_identity() {
    let keys: ContentKeys = [(ContentIdentity::new("abc"), ContentKey::new(*KEY))]
        .into_iter()
        .collect();
    assert_eq!(
        keys.key_for(&key_request("abc")),
        Some(ContentKey::new(*KEY))
    );
    assert_eq!(keys.key_for(&key_request("other")), None);
}

#[test]
fn a_provider_is_usable_as_a_trait_object() {
    let keys: std::sync::Arc<dyn ContentKeyProvider> = std::sync::Arc::new(ContentKeys::new());
    assert_eq!(keys.key_for(&key_request("abc")), None);
}

#[cfg(feature = "encrypted")]
mod sidecar {
    use std::fs;
    use std::path::Path;

    use bedrock_addon::storage::ContentKey;
    use bedrock_addon_std::keys::read_sidecar_key;

    use super::KEY;

    /// A temporary directory with `name` (the pack) and `name.key` holding `key`.
    fn pack_with_sidecar(name: &str, key: &[u8]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(name), b"PK\x03\x04").unwrap();
        fs::write(dir.path().join(format!("{name}.key")), key).unwrap();
        dir
    }

    #[test]
    fn reads_the_key_next_to_an_archive() {
        let dir = pack_with_sidecar("pack.mcpack", KEY);
        assert_eq!(
            read_sidecar_key(dir.path().join("pack.mcpack")),
            Some(ContentKey::new(*KEY))
        );
    }

    #[test]
    fn reads_the_key_next_to_a_pack_folder() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("ruby_pack")).unwrap();
        fs::write(dir.path().join("ruby_pack.key"), KEY).unwrap();
        assert_eq!(
            read_sidecar_key(dir.path().join("ruby_pack")),
            Some(ContentKey::new(*KEY))
        );
    }

    #[test]
    fn surrounding_ascii_whitespace_is_ignored() {
        let dir = pack_with_sidecar("pack.mcpack", b"0123456789abcdefghijklmnopqrstuv\r\n");
        assert_eq!(
            read_sidecar_key(dir.path().join("pack.mcpack")),
            Some(ContentKey::new(*KEY))
        );
    }

    #[test]
    fn a_missing_sidecar_is_no_key() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("pack.mcpack"), b"PK").unwrap();
        assert_eq!(read_sidecar_key(dir.path().join("pack.mcpack")), None);
    }

    #[test]
    fn a_key_of_the_wrong_length_is_no_key() {
        for text in [
            &b"0123456789abcdefghijklmnopqrstu"[..],
            b"0123456789abcdefghijklmnopqrstuvw",
            b"",
        ] {
            let dir = pack_with_sidecar("pack.mcpack", text);
            assert_eq!(
                read_sidecar_key(dir.path().join("pack.mcpack")),
                None,
                "{} bytes",
                text.len()
            );
        }
    }

    #[test]
    fn a_key_with_non_ascii_or_inner_whitespace_is_no_key() {
        for text in [
            "0123456789abcdefghijklmnopqrst\u{e9}".as_bytes(),
            b"0123456789abcdefghij lmnopqrstuv",
            b"0123456789abcdefghij\0lmnopqrstuv",
        ] {
            let dir = pack_with_sidecar("pack.mcpack", text);
            assert_eq!(
                read_sidecar_key(dir.path().join("pack.mcpack")),
                None,
                "{text:?}"
            );
        }
    }

    #[test]
    fn accepts_an_owned_path_and_a_str() {
        let dir = pack_with_sidecar("pack.mcpack", KEY);
        let owned = dir.path().join("pack.mcpack");
        assert_eq!(read_sidecar_key(owned.clone()), Some(ContentKey::new(*KEY)));
        assert_eq!(
            read_sidecar_key(owned.to_str().unwrap()),
            Some(ContentKey::new(*KEY))
        );
    }

    #[test]
    fn a_path_without_a_file_name_is_no_key() {
        assert_eq!(read_sidecar_key(Path::new("/")), None);
        assert_eq!(read_sidecar_key(Path::new("")), None);
    }

    #[test]
    fn a_sidecar_that_is_a_directory_is_no_key() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("pack.mcpack.key")).unwrap();
        assert_eq!(read_sidecar_key(dir.path().join("pack.mcpack")), None);
    }
}
