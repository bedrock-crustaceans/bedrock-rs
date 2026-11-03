//! The brarchive read-through layer: small archives written by the documented layout, the
//! malformed shapes, the `__brarchive/` path mapping, and every archive of the game's packs.

use bytes::Bytes;

use super::{fixture, game_files};
use crate::storage::brarchive::{Brarchive, BrarchiveLayer, archive_dir};
use crate::storage::{PackPath, PathRejected, StorageError};

const RUBY: &[u8] =
    br#"{"format_version":"1.21.0","minecraft:block":{"description":{"identifier":"demo:ruby"}}}"#;
const SAPPHIRE: &[u8] = br#"{"format_version":"1.21.0","scale":1.0}"#;

fn small_entries() -> Vec<(&'static str, &'static [u8])> {
    vec![
        ("ruby.json", RUBY),
        ("main.js", b""),
        ("sapphire.json", SAPPHIRE),
    ]
}

fn small() -> Bytes {
    Bytes::from(fixture("brarchive/small.brarchive"))
}

/// `(name, bytes)` of every entry, in archive order.
fn entries(archive: &Brarchive) -> Vec<(String, Bytes)> {
    archive
        .entries()
        .iter()
        .map(|entry| (entry.name.to_string(), archive.read(entry)))
        .collect()
}

mod fixtures {
    use super::*;

    #[test]
    fn the_brarchive_crate_writes_the_committed_bytes() {
        assert_eq!(brarchive::serialize(small_entries()).unwrap(), small());
        assert_eq!(
            brarchive::serialize(Vec::<(&str, &[u8])>::new()).unwrap(),
            fixture("brarchive/empty.brarchive")
        );
    }

    #[test]
    fn decodes_every_entry_in_order() {
        let archive = Brarchive::decode(small()).unwrap();
        let expected: Vec<(String, Bytes)> = small_entries()
            .into_iter()
            .map(|(name, bytes)| (name.to_owned(), Bytes::from_static(bytes)))
            .collect();
        assert_eq!(entries(&archive), expected);
    }

    #[test]
    fn names_agree_with_the_brarchive_crate() {
        let archive = Brarchive::decode(small()).unwrap();
        let names: Vec<&str> = archive.entries().iter().map(|entry| &*entry.name).collect();
        let bytes = small();
        let crate_names: Vec<&str> = brarchive::entries(&bytes)
            .unwrap()
            .map(|entry| entry.unwrap().name)
            .collect();
        assert_eq!(names, crate_names);
    }

    #[test]
    fn an_entry_is_a_slice_of_the_archive() {
        let bytes = small();
        let archive = Brarchive::decode(bytes.clone()).unwrap();
        let ruby = archive.get("ruby.json").unwrap();
        let start = bytes.as_ptr() as usize;
        let at = ruby.as_ptr() as usize;
        assert!(at >= start && at + ruby.len() <= start + bytes.len());
    }

    #[test]
    fn an_empty_archive_has_no_entries() {
        let archive = Brarchive::decode(Bytes::from(fixture("brarchive/empty.brarchive"))).unwrap();
        assert!(archive.entries().is_empty());
    }

    #[test]
    fn a_repeated_name_keeps_every_entry_and_reads_the_last() {
        let bytes =
            brarchive::serialize([("a.json", "1"), ("b.json", "2"), ("a.json", "3")]).unwrap();
        let archive = Brarchive::decode(Bytes::from(bytes)).unwrap();
        let names: Vec<&str> = archive.entries().iter().map(|entry| &*entry.name).collect();
        assert_eq!(names, ["a.json", "b.json", "a.json"]);
        assert_eq!(archive.get("a.json"), Some(Bytes::from_static(b"3")));
    }

    #[test]
    fn reencodes_to_the_same_entries() {
        let archive = Brarchive::decode(small()).unwrap();
        let again = Brarchive::decode(Bytes::from(archive.encode().unwrap())).unwrap();
        assert_eq!(entries(&again), entries(&archive));
    }
}

mod malformed {
    use super::*;

    fn refused(bytes: Vec<u8>) -> bool {
        matches!(
            Brarchive::decode(Bytes::from(bytes)),
            Err(StorageError::Brarchive)
        )
    }

    #[test]
    fn wrong_magic() {
        let mut bytes = small().to_vec();
        bytes[0] ^= 0xFF;
        assert!(refused(bytes));
    }

    #[test]
    fn wrong_version() {
        let mut bytes = small().to_vec();
        bytes[12] = 2;
        assert!(refused(bytes));
    }

    #[test]
    fn shorter_than_the_header() {
        assert!(refused(small()[..15].to_vec()));
        assert!(refused(Vec::new()));
    }

    #[test]
    fn descriptors_cut_short() {
        // the count promises three descriptors that the buffer does not hold
        assert!(refused(small()[..17].to_vec()));
        assert!(refused(small()[..16 + 256 * 2 + 10].to_vec()));
    }

    #[test]
    fn an_entry_count_far_past_the_data() {
        let mut bytes = small().to_vec();
        bytes[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(refused(bytes));
    }

    #[test]
    fn an_entry_past_the_end() {
        let bytes = small();
        assert!(refused(bytes[..bytes.len() - 1].to_vec()));
    }

    #[test]
    fn a_name_longer_than_247_bytes() {
        let mut bytes = small().to_vec();
        bytes[16] = 248;
        assert!(refused(bytes));
    }

    #[test]
    fn a_name_that_is_not_utf8() {
        let mut bytes = small().to_vec();
        bytes[17] = 0xFF;
        assert!(refused(bytes));
    }
}

mod paths {
    use super::*;

    fn dir(path: &'static str) -> Option<PackPath> {
        archive_dir(&PackPath::from_static(path))
    }

    #[test]
    fn an_archive_serves_the_folder_it_is_named_for() {
        assert_eq!(
            dir("__brarchive/items.brarchive"),
            Some(PackPath::from_static("items"))
        );
        assert_eq!(
            dir("__brarchive/trading/economy_trades.brarchive"),
            Some(PackPath::from_static("trading/economy_trades"))
        );
        assert_eq!(
            dir("subpacks/high/__brarchive/textures.brarchive"),
            Some(PackPath::from_static("subpacks/high/textures"))
        );
    }

    #[test]
    fn only_brarchive_files_under_the_folder_count() {
        assert_eq!(dir("items.brarchive"), None);
        assert_eq!(dir("__brarchive/notes.txt"), None);
        assert_eq!(dir("__brarchive"), None);
    }

    #[test]
    fn an_entry_name_breaking_the_path_rules_is_rejected() {
        let bytes = brarchive::serialize([("../escape.json", "{}")]).unwrap();
        let archive = Brarchive::decode(Bytes::from(bytes)).unwrap();
        let mut layer = BrarchiveLayer::default();
        assert!(matches!(
            layer.add(&PackPath::from_static("items"), &archive),
            Err(StorageError::PathRejected {
                reason: PathRejected::DotSegment { .. },
                ..
            })
        ));
    }
}

mod layer {
    use super::*;

    fn layer() -> BrarchiveLayer {
        let mut layer = BrarchiveLayer::default();
        let dir = archive_dir(&PackPath::from_static("__brarchive/blocks.brarchive")).unwrap();
        layer
            .add(&dir, &Brarchive::decode(small()).unwrap())
            .unwrap();
        layer
    }

    #[test]
    fn serves_entries_under_the_archive_folder() {
        assert_eq!(
            layer().get(&PackPath::from_static("blocks/ruby.json")),
            Some(Bytes::from_static(RUBY))
        );
        assert_eq!(layer().get(&PackPath::from_static("ruby.json")), None);
    }

    #[test]
    fn a_zero_size_entry_does_not_shadow_the_loose_file() {
        // the game's library packs ship `scripts.brarchive` with every entry empty next to
        // the loose `scripts/*.js`
        let layer = layer();
        let main = PackPath::from_static("blocks/main.js");
        assert!(layer.contains(&main));
        assert_eq!(layer.get(&main), None);
    }

    #[test]
    fn lists_every_entry_path_sorted() {
        let paths: Vec<String> = layer().paths().map(ToString::to_string).collect();
        assert_eq!(
            paths,
            ["blocks/main.js", "blocks/ruby.json", "blocks/sapphire.json"]
        );
    }

    #[test]
    fn a_later_archive_wins_for_the_same_path() {
        let mut layer = layer();
        let other = brarchive::serialize([("ruby.json", "{}")]).unwrap();
        layer
            .add(
                &PackPath::from_static("blocks"),
                &Brarchive::decode(Bytes::from(other)).unwrap(),
            )
            .unwrap();
        assert_eq!(
            layer.get(&PackPath::from_static("blocks/ruby.json")),
            Some(Bytes::from_static(b"{}"))
        );
    }
}

/// Every `.brarchive` of the game's installs: the layer lists the names the
/// `brarchive` crate lists, reads the bytes it reads, and an archive written back from the
/// entries decodes to the same entries.
#[test]
fn every_vanilla_archive_round_trips() {
    let Some(archives) = game_files(&["bds-1.26.36.1", "gdk-retail"], ".brarchive") else {
        return;
    };
    assert!(!archives.is_empty());
    let mut entry_count = 0;
    for path in &archives {
        let bytes = Bytes::from(std::fs::read(path).unwrap());
        let archive = Brarchive::decode(bytes.clone())
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let oracle: Vec<(String, Vec<u8>)> = brarchive::deserialize(&bytes).unwrap();
        let read: Vec<(String, Vec<u8>)> = entries(&archive)
            .into_iter()
            .map(|(name, bytes)| (name, bytes.to_vec()))
            .collect();
        assert_eq!(read, oracle, "{}", path.display());
        let again = Brarchive::decode(Bytes::from(archive.encode().unwrap())).unwrap();
        assert_eq!(entries(&again), entries(&archive), "{}", path.display());
        entry_count += oracle.len();
    }
    eprintln!("{} archives, {entry_count} entries", archives.len());
}
