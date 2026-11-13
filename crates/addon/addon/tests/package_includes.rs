//! The package-include rail of `bedrock_addon`: which of its test files include a file
//! outside the package (`bedrock_addon_test_support::package` says what is seen and why it
//! matters).

use bedrock_addon_test_support::package::{includes_outside_the_package, rust_files};

/// The test files of this package that include a file outside it, and are known to, as
/// `"<file>: <path>"`. Every family is the crate `bedrock_addon_family_<name>` under
/// `crates/addon/families/`, and its test helpers (`common.rs`, `generate.rs`, `compare.rs`) and
/// its component list live there with it. The tests of `bedrock_addon` that build a `PackStack`
/// of the family include those helpers by `#[path]` rather than keep a second copy, so
/// `bedrock_addon`'s tests build in the workspace, where every family is a sibling. A new
/// include joins the list only when it reaches a family's `tests/<name>/` helper or its
/// component list.
const KNOWN_OUTSIDE: &[&str] = &[
    "tests/atomic_client_entity.rs: ../../families/client_entity/tests/client_entity/generate.rs",
    "tests/attachable.rs: ../../families/attachable/tests/attachable/common.rs",
    "tests/biome.rs: ../../families/biome/tests/biome/common.rs",
    "tests/block_culling.rs: ../../families/block_culling/tests/block_culling/common.rs",
    "tests/camera.rs: ../../families/camera/tests/camera/common.rs",
    "tests/client_entity.rs: ../../families/client_entity/tests/client_entity/common.rs",
    "tests/crafting_catalog.rs: ../../families/crafting_catalog/tests/crafting_catalog/common.rs",
    "tests/deferred_settings.rs: ../../families/deferred_settings/tests/deferred_settings/common.rs",
    "tests/dimension.rs: ../../families/dimension/tests/dimension/common.rs",
    "tests/feature.rs: ../../families/feature/tests/feature/common.rs",
    "tests/feature_rules.rs: ../../families/feature_rules/tests/feature_rules/common.rs",
    "tests/item.rs: ../../families/item/tests/item/common.rs",
    "tests/loot.rs: ../../families/loot/tests/loot/common.rs",
    "tests/material.rs: ../../families/material/tests/material/common.rs",
    "tests/persona.rs: ../../families/persona/tests/persona/common.rs",
    "tests/recipe.rs: ../../families/recipe/tests/recipe/common.rs",
    "tests/script_bundle.rs: ../../families/script/tests/script_bundle/common.rs",
    "tests/script_module.rs: ../../families/script/tests/script_module/common.rs",
    "tests/server_sound_definitions.rs: ../../families/server_sound_definitions/tests/server_sound_definitions/common.rs",
    "tests/spawn_groups.rs: ../../families/spawn_rules/tests/spawn_rules/common.rs",
    "tests/spawn_rules.rs: ../../families/spawn_rules/tests/spawn_rules/common.rs",
    "tests/structure.rs: ../../families/structure/tests/structure/common.rs",
];

/// This package builds from its own files: no test file includes one of another crate but the
/// ones [`KNOWN_OUTSIDE`] lists.
#[test]
fn no_test_file_of_this_package_includes_a_file_outside_it() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(root, &root.join("tests"), &mut files);
    let mut found: Vec<String> = files
        .iter()
        .flat_map(|(file, text)| includes_outside_the_package(file, text))
        .collect();
    found.sort();
    let mut known: Vec<&str> = KNOWN_OUTSIDE.to_vec();
    known.sort_unstable();
    assert_eq!(
        found,
        known,
        "includes that leave {}",
        env!("CARGO_PKG_NAME")
    );
    // The file this test is in is one of the package's.
    assert!(
        files
            .iter()
            .any(|(file, _)| file == "tests/package_includes.rs"),
        "the walk finds this file"
    );
}
