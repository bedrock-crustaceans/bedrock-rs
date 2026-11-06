//! The public face of `storage`: pack paths, content keys and the storage error. The container
//! formats themselves are internal to `bedrock_addon_storage` and tested beside their code
//! there (`crates/addon/storage/src/tests/`).

use bedrock_addon::storage::{
    ContentIdentity, ContentKey, ContentKeys, PackPath, PathRejected, StorageError,
};

const KEY_TEXT: &[u8; 32] = b"0123456789abcdef0123456789ABCDEF";

mod content_key {
    use super::*;

    #[test]
    fn debug_never_prints_the_key() {
        let key = ContentKey::new(*KEY_TEXT);
        assert_eq!(format!("{key:?}"), "ContentKey(..)");
        assert_eq!(format!("{key:#?}"), "ContentKey(..)");
    }

    #[test]
    fn from_slice_takes_the_first_32_bytes() {
        let mut long = KEY_TEXT.to_vec();
        long.extend_from_slice(b"ignored tail");
        assert_eq!(
            ContentKey::from_slice(&long),
            Some(ContentKey::new(*KEY_TEXT))
        );
    }

    #[test]
    fn from_slice_refuses_a_short_key() {
        assert_eq!(ContentKey::from_slice(&KEY_TEXT[..31]), None);
    }
}

mod content_identity {
    use super::*;

    #[test]
    fn displays_the_header_string() {
        let identity = ContentIdentity::new("325FCAEA-29F0-4D51-AB14-A0BE8C66DC97");
        assert_eq!(identity.to_string(), "325FCAEA-29F0-4D51-AB14-A0BE8C66DC97");
        assert_eq!(identity.as_str(), "325FCAEA-29F0-4D51-AB14-A0BE8C66DC97");
    }
}

mod content_keys {
    use super::*;

    #[test]
    fn looks_a_key_up_by_identity() {
        let identity = ContentIdentity::new("pack-a");
        let mut keys = ContentKeys::new();
        assert!(keys.is_empty());
        assert_eq!(
            keys.insert(identity.clone(), ContentKey::new(*KEY_TEXT)),
            None
        );
        assert_eq!(keys.get(&identity), Some(&ContentKey::new(*KEY_TEXT)));
        assert_eq!(keys.get(&ContentIdentity::new("pack-b")), None);
        assert_eq!(keys.len(), 1);
    }

    #[test]
    fn collects_from_pairs_and_debug_hides_the_keys() {
        let keys: ContentKeys = [(ContentIdentity::new("pack-a"), ContentKey::new(*KEY_TEXT))]
            .into_iter()
            .collect();
        let debug = format!("{keys:?}");
        assert!(debug.contains("ContentKey(..)"), "{debug}");
        assert!(!debug.contains("0123456789abcdef"), "{debug}");
    }
}

mod storage_error {
    use super::*;

    #[test]
    fn path_rejected_carries_the_written_path_and_core_reason() {
        let reason = PackPath::new("../escape.json").unwrap_err();
        let error = StorageError::PathRejected {
            path: "../escape.json".into(),
            reason,
        };
        assert!(matches!(
            error,
            StorageError::PathRejected {
                reason: PathRejected::DotSegment { index: 0 },
                ..
            }
        ));
    }
}
