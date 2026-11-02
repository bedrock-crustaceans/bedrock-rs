#![allow(clippy::pedantic, missing_docs)]
//! The pack-identity vocabulary as a user of `bedrock_addon_core::pack_identity` sees it:
//! `PackPath` under the storage rules, `PackIdVersion` equality, the
//! `PackRef` handle and the vanilla enums.

use std::collections::HashSet;

use bedrock_addon_core::pack_identity::{
    PackIdVersion, PackOrigin, PackPath, PackRef, PackSide, PackType, PathRejected,
};
use bedrock_addon_core::version::SemVersion;
use uuid::Uuid;

fn path(text: &str) -> PackPath {
    PackPath::new(text).unwrap_or_else(|e| panic!("{text:?} is a pack path: {e}"))
}

fn segments(path: &PackPath) -> Vec<&str> {
    path.segments().collect()
}

mod pack_path {
    use super::*;

    #[test]
    fn a_plain_path_keeps_its_segments() {
        let p = path("blocks/ruby.json");
        assert_eq!(segments(&p), ["blocks", "ruby.json"]);
        assert_eq!(p.as_str(), "blocks/ruby.json");
        assert_eq!(p.to_string(), "blocks/ruby.json");
    }

    #[test]
    fn backslashes_and_repeated_separators_normalise_to_one_slash() {
        let p = path("\\textures\\\\blocks//stone.png/");
        assert_eq!(p.as_str(), "textures/blocks/stone.png");
        assert_eq!(p, path("textures/blocks/stone.png"));
    }

    #[test]
    fn the_empty_path_is_the_root() {
        assert_eq!(path(""), PackPath::ROOT);
        assert_eq!(path("//"), PackPath::ROOT);
        assert!(PackPath::ROOT.is_root());
        assert_eq!(segments(&PackPath::ROOT), Vec::<&str>::new());
        assert_eq!(PackPath::ROOT.to_string(), "");
    }

    #[test]
    fn paths_are_case_sensitive() {
        assert_ne!(path("Blocks/ruby.json"), path("blocks/ruby.json"));
    }

    #[test]
    fn dot_segments_are_rejected() {
        assert_eq!(
            PackPath::new("blocks/../manifest.json"),
            Err(PathRejected::DotSegment { index: 7 })
        );
        assert_eq!(
            PackPath::new("./manifest.json"),
            Err(PathRejected::DotSegment { index: 0 })
        );
        assert_eq!(
            PackPath::new("a\\..\\b"),
            Err(PathRejected::DotSegment { index: 2 })
        );
        assert!(
            PackPath::new("a/.hidden/..b/c.").is_ok(),
            "only whole `.` / `..` segments"
        );
    }

    #[test]
    fn deviation_control_characters_and_colons_are_rejected() {
        assert_eq!(
            PackPath::new("a\0b"),
            Err(PathRejected::InvalidChar { index: 1, ch: '\0' })
        );
        assert_eq!(
            PackPath::new("C:/Windows"),
            Err(PathRejected::InvalidChar { index: 1, ch: ':' })
        );
        assert_eq!(
            PackPath::new("file.json:stream"),
            Err(PathRejected::InvalidChar { index: 9, ch: ':' })
        );
    }

    #[test]
    fn more_than_64_segments_are_rejected() {
        let sixty_four = vec!["a"; 64].join("/");
        assert_eq!(segments(&path(&sixty_four)).len(), 64);
        let sixty_five = vec!["a"; 65].join("/");
        assert_eq!(
            PackPath::new(&sixty_five),
            Err(PathRejected::TooManySegments)
        );
    }

    #[test]
    fn more_than_1023_bytes_are_rejected() {
        let at_limit = format!("{}/{}", "a".repeat(511), "b".repeat(511));
        assert_eq!(path(&at_limit).as_str().len(), 1023);
        let over = format!("{}/{}", "a".repeat(512), "b".repeat(511));
        assert_eq!(PackPath::new(&over), Err(PathRejected::TooLong));
        // The limit is on the normalised path: collapsed separators do not count.
        let padded = format!("{}//////{}", "a".repeat(511), "b".repeat(511));
        assert!(PackPath::new(&padded).is_ok());
    }

    #[test]
    fn non_ascii_names_are_kept() {
        let p = path("texts/日本語.lang");
        assert_eq!(segments(&p), ["texts", "日本語.lang"]);
    }

    #[test]
    fn a_static_path_is_validated() {
        assert_eq!(
            PackPath::from_static("blocks\\ruby.json"),
            path("blocks/ruby.json")
        );
    }

    #[test]
    #[should_panic(expected = "pack path")]
    fn a_malformed_static_path_panics() {
        let _ = PackPath::from_static("../manifest.json");
    }

    #[test]
    fn join_appends_a_validated_path() {
        let root = path("subpacks/high");
        assert_eq!(
            root.join(&path("blocks/x.json")),
            path("subpacks/high/blocks/x.json")
        );
        assert_eq!(PackPath::ROOT.join(&path("a")), path("a"));
        assert_eq!(path("a").join(&PackPath::ROOT), path("a"));
    }

    #[test]
    fn paths_order_bytewise() {
        let mut paths = [path("b"), path("a/b"), path("a-b"), path("A")];
        paths.sort();
        let sorted: Vec<&str> = paths.iter().map(PackPath::as_str).collect();
        assert_eq!(sorted, ["A", "a-b", "a/b", "b"]);
    }

    #[test]
    fn a_rejection_renders_its_rule() {
        assert_eq!(
            PathRejected::DotSegment { index: 7 }.to_string(),
            "`.` or `..` segment at byte 7"
        );
    }
}

mod identity {
    use super::*;

    fn uuid() -> Uuid {
        Uuid::parse_str("fe9f8597-5454-481a-8730-8d070a8e2e58").unwrap()
    }

    #[test]
    fn pack_id_version_equality_is_uuid_and_version() {
        let a = PackIdVersion::new(uuid(), SemVersion::new(1, 0, 0), PackType::Behavior);
        let b = PackIdVersion::new(uuid(), SemVersion::new(1, 0, 0), PackType::Resource);
        assert_eq!(a, b, "the pack type takes no part");
        let set: HashSet<_> = [a.clone(), b].into_iter().collect();
        assert_eq!(set.len(), 1);
        assert_ne!(
            a,
            PackIdVersion::new(uuid(), SemVersion::new(1, 0, 1), PackType::Behavior)
        );
        assert_ne!(
            a,
            PackIdVersion::new(Uuid::nil(), SemVersion::new(1, 0, 0), PackType::Behavior)
        );
    }

    #[test]
    fn pack_id_version_shows_uuid_and_version() {
        let id = PackIdVersion::new(uuid(), SemVersion::new(0, 0, 1), PackType::Behavior);
        assert_eq!(id.to_string(), "fe9f8597-5454-481a-8730-8d070a8e2e58 0.0.1");
    }

    #[test]
    fn a_pack_ref_carries_index_identity_and_name() {
        let id = PackIdVersion::new(uuid(), SemVersion::new(0, 0, 1), PackType::Behavior);
        let pack = PackRef::new(2, id.clone(), "vanilla");
        assert_eq!(pack.name(), "vanilla");
        assert_eq!(pack.index(), Some(2));
        assert_eq!(pack.id(), &id);
        assert!(!pack.is_app_data());
        assert_eq!(pack.clone(), pack);
        assert_ne!(pack, PackRef::new(3, id, "vanilla"));
    }

    #[test]
    fn the_app_data_ref_is_no_stack_entry() {
        let app = PackRef::app_data();
        assert!(app.is_app_data());
        assert_eq!(app.index(), None);
        assert_eq!(app, PackRef::app_data());
        assert_eq!(app.id().uuid, Uuid::nil());
    }
}

mod enums {
    use super::*;

    #[test]
    fn pack_types_keep_vanilla_values() {
        let table = [
            (PackType::Invalid, 0),
            (PackType::Addon, 1),
            (PackType::Cached, 2),
            (PackType::CopyProtected, 3),
            (PackType::Behavior, 4),
            (PackType::PersonaPiece, 5),
            (PackType::Resource, 6),
            (PackType::Skins, 7),
            (PackType::WorldTemplate, 8),
        ];
        for (pack_type, value) in table {
            assert_eq!(pack_type as i8, value, "{pack_type:?}");
        }
    }

    #[test]
    fn pack_origins_keep_vanilla_values_and_only_package_is_base_game() {
        let table = [
            (PackOrigin::Unknown, 0),
            (PackOrigin::RealmsUnknown, 1),
            (PackOrigin::Package, 2),
            (PackOrigin::Treatment, 3),
            (PackOrigin::Dev, 4),
            (PackOrigin::World, 5),
            (PackOrigin::User, 6),
            (PackOrigin::TempCache, 7),
            (PackOrigin::PremiumCache, 8),
            (PackOrigin::PremiumTempCache, 9),
            (PackOrigin::RealmsService, 10),
            (PackOrigin::Test, 11),
            (PackOrigin::System, 12),
        ];
        for (origin, value) in table {
            assert_eq!(origin as u8, value, "{origin:?}");
            assert_eq!(
                origin.is_base_game(),
                origin == PackOrigin::Package,
                "{origin:?}"
            );
        }
    }

    #[test]
    fn pack_sides_name_the_two_stacks() {
        assert_ne!(PackSide::Behavior, PackSide::Resource);
        assert_eq!(PackSide::Behavior.pack_type(), PackType::Behavior);
        assert_eq!(PackSide::Resource.pack_type(), PackType::Resource);
    }
}
