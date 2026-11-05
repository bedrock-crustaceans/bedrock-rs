//! Unit tests of what the public API cannot see: when a loaded structure keeps its file's tree.

use bedrock_addon_core::load::LoadContext;

use super::read;

/// The structure of `bytes`, which must load.
fn load(bytes: &[u8]) -> super::Structure {
    read::read(bytes, &LoadContext::detached())
        .0
        .expect("the structure loads")
}

#[test]
fn a_file_in_vanillas_layout_keeps_no_tree() {
    for bytes in [
        &include_bytes!(
            "../../../addon/tests/fixtures/structure/structure/capture_sulfur_spring_small_1.mcstructure"
        )[..],
        &include_bytes!(
            "../../../addon/tests/fixtures/structure/structure/capture_processors_test.mcstructure"
        )[..],
        &include_bytes!(
            "../../../addon/tests/fixtures/structure/structure/capture_hoglin.mcstructure"
        )[..],
    ] {
        let structure = load(bytes);
        assert!(structure.layout.source.is_none());
        assert!(structure.layout.trailing.is_empty());
    }
}

#[test]
fn a_file_in_another_layout_keeps_its_tree() {
    // Tags out of `std::map` order and an empty entity list of Compound tags.
    let structure = load(include_bytes!(
        "../../../addon/tests/fixtures/structure/structure/capture_tick_block_structure.mcstructure"
    ));
    assert!(structure.layout.source.is_some());
}

#[test]
fn a_rejected_copy_in_a_subpack_is_keyed_without_its_folder() {
    use bedrock_addon_core::pack_identity::PackPath;
    let key = PackPath::from_static("structures/house.mcstructure");
    let keyed = |path: &'static str| crate::keyed(&PackPath::from_static(path), &key);
    assert!(keyed("structures/house.mcstructure"));
    assert!(keyed("subpacks/high/structures/house.mcstructure"));
    assert!(!keyed("subpacks/highstructures/house.mcstructure"));
    assert!(!keyed("other/structures/house.mcstructure"));
    assert!(!keyed("structures/mystructure/house.mcstructure"));
}
