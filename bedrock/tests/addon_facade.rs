#![cfg(feature = "addon-std")]

use bedrock::addon::pack::{OpenOptions, Pack, PackRepository, PackType};
use bedrock::addon::storage::PackPath;

const BEHAVIOR_MANIFEST: &[u8] = br#"{
    "format_version": 2,
    "header": {
        "name": "facade pack",
        "description": "",
        "uuid": "00000000-0000-0000-0000-000000000001",
        "version": [1, 0, 0],
        "min_engine_version": [1, 21, 0]
    },
    "modules": [
        { "type": "data", "uuid": "00000000-0000-0000-0000-000000000002", "version": [1, 0, 0] }
    ]
}"#;

#[test]
fn both_addon_crates_are_named_through_the_facade() {
    let _repository = PackRepository::new();
    let options = bedrock::addon_std::Options::default();
    assert!(
        options.keys.is_none(),
        "the default driver options hold no key provider"
    );
}

#[test]
fn a_pack_opens_from_in_memory_bytes_through_the_facade() {
    let pack = Pack::from_files(
        [(PackPath::from_static("manifest.json"), BEHAVIOR_MANIFEST)],
        &OpenOptions::default(),
    )
    .expect("the pack opens");
    assert_eq!(pack.pack_type(), PackType::Behavior);
}
