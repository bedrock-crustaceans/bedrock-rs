//! The fixture rows whose expected finding is a 20 row with When = `link` (M7: a `blocks` entry
//! naming no block, the validation rules X1). They run through `StackSet::link()`; the finding itself
//! is also pinned on the crate-private checker in `src/poi/tests.rs`, and what the load does
//! with those documents in `stack.rs`.

// (each rule of this file relates its module and `block`)
#![cfg(feature = "block")]

use std::sync::Arc;

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticDetail, Diagnostics, Severity};
use bedrock_addon::load::BuiltIns;
use bedrock_addon::pack::{PackOrigin, PackRepository, WorldPacks};

use super::common::{fixture_packs, poi_options};

/// The vanilla palette of these rows: every `minecraft:` block is built in.
struct VanillaBlocks;

impl BuiltIns for VanillaBlocks {
    fn contains(&self, kind: &'static str, id: &str) -> bool {
        kind == "block" && id.starts_with("minecraft:")
    }
}

/// The link findings of a fixture's stack, the `poi` experiment on and the vanilla blocks
/// built in.
fn link_fixture(name: &str) -> Diagnostics {
    let packs = fixture_packs(name);
    let entries: Vec<String> = packs
        .iter()
        .map(|pack| {
            format!(
                r#"{{"pack_id": "{}", "version": [1, 0, 0]}}"#,
                pack.id().uuid
            )
        })
        .collect();
    let world = WorldPacks::parse(Some(format!("[{}]", entries.join(",")).as_bytes()), None)
        .expect("a world list");
    let mut repository = PackRepository::new();
    for pack in packs {
        repository.add_pack(pack, PackOrigin::User);
    }
    let mut options = poi_options();
    options.built_ins = Some(Arc::new(VanillaBlocks));
    repository.stacks(&world, options).link()
}

/// The unresolved block references of `d`, as `(JSON path, block)`.
fn unresolved_blocks(d: &Diagnostics) -> Vec<(String, String)> {
    d.iter()
        .filter(|f: &&Diagnostic| {
            f.code.id() == "poi/block_lookup_failed" && f.severity == Severity::Error
        })
        .filter_map(|f| match f.detail.as_deref() {
            Some(DiagnosticDetail::Reference {
                kind: "block", id, ..
            }) => Some((f.location.json_path.to_string(), id.clone())),
            _ => None,
        })
        .collect()
}

#[test]
fn poi31_unknown_block_is_reported_and_the_known_block_is_not() {
    let d = link_fixture("poi31_unknown_block");
    assert_eq!(
        unresolved_blocks(&d),
        [(
            "minecraft:poi_block/blocks/0".to_owned(),
            "test:not_a_block".to_owned()
        )],
        "{d:?}"
    );
}

#[test]
fn poi40_all_blocks_unknown_is_reported_without_a_no_blocks_warning() {
    let d = link_fixture("poi40_all_blocks_unknown");
    assert_eq!(
        unresolved_blocks(&d),
        [(
            "minecraft:poi_block/blocks/0".to_owned(),
            "test:not_a_block".to_owned()
        )],
        "{d:?}"
    );
    assert!(
        d.iter().all(|f| f.code.id() != "core/poi_no_blocks"),
        "{d:?}"
    );
}
