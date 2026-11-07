//! 20 M7: the blocks of a point-of-interest type. M8 (the tags of a type, the edges of a tag)
//! is the load's and is audited in `audit.rs`.

#![allow(unused_imports, reason = "which rows compile depends on the features")]

use bedrock_addon::load::{Experiment, Experiments, LoadOptions};
use bedrock_addon::pack::LinkOptions;

use super::super::common::{hosted, link_with};
#[cfg(feature = "block")]
use super::{block, finding, said, unresolved_warning};
#[cfg(feature = "block")]
use bedrock_addon::diagnostic::Severity;

/// A POI type `demo:forge` whose blocks are `blocks` (a JSON array).
fn forge(blocks: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.40", "minecraft:poi_block": {{"description": {{"identifier": "demo:forge"}}, "blocks": {blocks}}}}}"#
    )
}

/// `options` with the experiment POI documents load under.
fn with_poi(mut options: LoadOptions) -> LoadOptions {
    let mut experiments = Experiments::new();
    experiments.insert(Experiment::Poi);
    options.experiments = experiments;
    options
}

#[cfg(feature = "block")]
#[test]
fn m7_a_block_of_a_poi_type_that_a_pack_defines_or_the_game_has_resolves() {
    let lamp = block("demo:lamp");
    let forge = forge(r#"["demo:lamp", "minecraft:anvil"]"#);
    let files = [
        ("blocks/lamp.json", lamp.as_str()),
        ("poi/blocks/forge.json", forge.as_str()),
    ];
    // Without a host the crate's list of the game's blocks answers for `minecraft:anvil`.
    let found = link_with(
        &files,
        &[],
        with_poi(LoadOptions::default()),
        LinkOptions::default(),
    );
    assert_eq!(said(&found, &["block"]), []);
    // A host answers for it too.
    let found = link_with(
        &files,
        &[],
        with_poi(hosted(&[("block", "minecraft:anvil")])),
        LinkOptions::default(),
    );
    assert_eq!(said(&found, &["block"]), []);
}

#[cfg(feature = "block")]
#[test]
fn m7_a_block_of_a_poi_type_that_nothing_defines_is_an_error_at_its_entry() {
    let forge = forge(r#"["minecraft:anvil", "demo:lamp", "minecraft:stone"]"#);
    let files = [("poi/blocks/forge.json", forge.as_str())];
    let found = link_with(
        &files,
        &[],
        with_poi(LoadOptions::default()),
        LinkOptions::default(),
    );
    assert_eq!(
        said(&found, &["block"]),
        // The registry's log is error level; the type keeps its other blocks.
        [finding(
            "poi/block_lookup_failed",
            Severity::Error,
            "poi/blocks/forge.json",
            "minecraft:poi_block/blocks/1",
            "block",
            "demo:lamp"
        )]
    );
    // An error is no lint: it stays with the lints off.
    let found = link_with(
        &files,
        &[],
        with_poi(LoadOptions::default()),
        LinkOptions {
            lints: false,
            ..Default::default()
        },
    );
    assert_eq!(said(&found, &["block"]).len(), 1);
    // A host that does not have a block of the game is believed.
    let found = link_with(
        &files,
        &[],
        with_poi(hosted(&[("block", "minecraft:anvil")])),
        LinkOptions::default(),
    );
    assert_eq!(
        said(&found, &["block"])
            .iter()
            .map(|said| said.5.as_str())
            .collect::<Vec<_>>(),
        ["demo:lamp", "minecraft:stone"]
    );
}

/// Without the experiment no POI document loads, so none is linked.
#[cfg(feature = "block")]
#[test]
fn m7_without_the_poi_experiment_nothing_is_linked() {
    let forge = forge(r#"["demo:lamp"]"#);
    let found = link_with(
        &[("poi/blocks/forge.json", forge.as_str())],
        &[],
        LoadOptions::default(),
        LinkOptions::default(),
    );
    assert_eq!(said(&found, &["block"]), []);
}

/// Without the `block` module the rule is not compiled: nobody knows a pack's blocks.
#[cfg(not(feature = "block"))]
#[test]
fn m7_without_the_block_module_no_block_of_a_poi_type_is_reported() {
    let forge = forge(r#"["demo:lamp"]"#);
    let found = link_with(
        &[("poi/blocks/forge.json", forge.as_str())],
        &[],
        with_poi(hosted(&[])),
        &LinkOptions::default(),
    );
    assert!(found.is_empty(), "{found:?}");
}
