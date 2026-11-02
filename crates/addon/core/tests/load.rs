//! `load`'s plain data: the virtual locations loaders speak in, the
//! default limits and in-memory app data.

use bedrock_addon_core::load::{AppData, Limits, Place, RootId};
use bedrock_addon_core::pack_identity::PackPath;

const MIB: u64 = 1024 * 1024;
const GIB: u64 = 1024 * MIB;

#[test]
fn limits_default() {
    assert_eq!(
        Limits::default(),
        Limits {
            max_entries: 65_536,
            max_entry_bytes: 256 * MIB,
            max_pack_bytes: GIB,
            // An archive is bounded like a pack: by the per-pack limit.
            max_archive_bytes: GIB,
            max_pending_reads: 64,
            max_pending_read_bytes: 256 * MIB,
            max_pending_jobs: 4096,
        }
    );
}

#[test]
fn memory_root_is_reserved_at_the_top_of_the_range() {
    assert_eq!(RootId::MEMORY, RootId(u32::MAX));
    assert!(RootId(0) < RootId::MEMORY);
}

#[test]
fn place_displays_as_root_and_path() {
    let place = Place {
        root: RootId(3),
        path: PackPath::from_static("blocks/ruby.json"),
    };
    assert_eq!(place.to_string(), "#3/blocks/ruby.json");
}

#[test]
fn a_root_place_displays_without_a_path() {
    let place = Place::root(RootId(3));
    assert!(place.path.is_root());
    assert_eq!(place.root, RootId(3));
    assert_eq!(place.to_string(), "#3");
}

#[test]
fn place_join_appends_below_the_path() {
    let pack = Place::root(RootId(7)).join(&PackPath::from_static("my_pack"));
    assert_eq!(pack.path.as_str(), "my_pack");
    let file = pack.join(&PackPath::from_static("blocks/ruby.json"));
    assert_eq!(file.root, RootId(7));
    assert_eq!(file.path.as_str(), "my_pack/blocks/ruby.json");
    assert_eq!(file.to_string(), "#7/my_pack/blocks/ruby.json");
    // Joining the empty path is the place itself.
    assert_eq!(file.join(&PackPath::ROOT), file);
}

#[test]
fn app_data_directories_are_the_four_definitions_folders() {
    // The families attachable, feature, feature_rules and
    // spawn_groups place them under the app's `definitions/` folder.
    assert_eq!(
        AppData::DIRECTORIES,
        [
            "definitions/attachables",
            "definitions/features",
            "definitions/feature_rules",
            "definitions/spawn_groups",
        ]
    );
}

#[test]
fn app_data_from_files_keeps_every_file_sorted_bytewise() {
    let data = AppData::from_files([
        (
            PackPath::from_static("definitions/spawn_groups/raid_group_2.json"),
            &b"two"[..],
        ),
        (
            PackPath::from_static("definitions/attachables/bow.json"),
            &b"bow"[..],
        ),
        (
            PackPath::from_static("definitions/spawn_groups/raid_group_1.json"),
            &b"one"[..],
        ),
    ]);
    assert_eq!(data.len(), 3);
    let paths: Vec<&str> = data.iter().map(|(path, _)| path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "definitions/attachables/bow.json",
            "definitions/spawn_groups/raid_group_1.json",
            "definitions/spawn_groups/raid_group_2.json",
        ]
    );
    assert_eq!(
        data.get(&PackPath::from_static("definitions/attachables/bow.json"))
            .map(|bytes| &bytes[..]),
        Some(&b"bow"[..])
    );
}

#[test]
fn app_data_files_under_a_directory_are_its_descendants_only() {
    let data = AppData::from_files([
        (
            PackPath::from_static("definitions/features/a.json"),
            &b"a"[..],
        ),
        (
            PackPath::from_static("definitions/features/sub/b.json"),
            &b"b"[..],
        ),
        (
            PackPath::from_static("definitions/feature_rules/c.json"),
            &b"c"[..],
        ),
    ]);
    let under: Vec<&str> = data
        .files_under("definitions/features")
        .map(|(path, _)| path.as_str())
        .collect();
    assert_eq!(
        under,
        [
            "definitions/features/a.json",
            "definitions/features/sub/b.json"
        ]
    );
}

#[test]
fn app_data_from_files_keeps_the_last_copy_of_a_path() {
    let data = AppData::from_files([
        (
            PackPath::from_static("definitions/features/a.json"),
            vec![1],
        ),
        (
            PackPath::from_static("definitions\\features/a.json"),
            vec![2],
        ),
    ]);
    assert_eq!(data.len(), 1);
    assert_eq!(
        data.get(&PackPath::from_static("definitions/features/a.json"))
            .map(|bytes| bytes.to_vec()),
        Some(vec![2])
    );
}

#[test]
fn empty_app_data_is_the_default() {
    let data = AppData::default();
    assert!(data.is_empty());
    assert_eq!(data.iter().count(), 0);
}

/// A JSON document larger than the game's limit of 128 MiB is refused; from 3·2²⁵ bytes (96 MiB) on
/// it is approaching the limit. Classified by length alone, so nothing large is allocated.
#[test]
fn a_json_document_is_sized_against_the_game_s_limits() {
    use bedrock_addon_core::load::{
        JSON_DOCUMENT_WARNING_BYTES, JsonDocumentSize, MAX_JSON_DOCUMENT_BYTES, json_document_size,
    };
    assert_eq!(MAX_JSON_DOCUMENT_BYTES, 134_217_728);
    assert_eq!(JSON_DOCUMENT_WARNING_BYTES, 100_663_296);
    assert_eq!(json_document_size(0), JsonDocumentSize::Within);
    assert_eq!(
        json_document_size(JSON_DOCUMENT_WARNING_BYTES - 1),
        JsonDocumentSize::Within
    );
    assert_eq!(
        json_document_size(JSON_DOCUMENT_WARNING_BYTES),
        JsonDocumentSize::Approaching
    );
    assert_eq!(
        json_document_size(MAX_JSON_DOCUMENT_BYTES),
        JsonDocumentSize::Approaching,
        "a document of exactly the limit is allowed; one byte more is not"
    );
    assert_eq!(
        json_document_size(MAX_JSON_DOCUMENT_BYTES + 1),
        JsonDocumentSize::TooLarge
    );
}
