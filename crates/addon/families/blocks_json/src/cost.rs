//! The cost of the places of this family that look a thing up per entry, each measured on its own
//! on inputs built beforehand, by the shared cost-test helper (`bedrock_addon_core::test_cost`):
//! four times the input takes about four times the time. A load through a stack cannot show this,
//! because the walk and the parse dominate it.

use super::load::{ParsedBlocks, combine_blocks};
use super::resolve::{self, duplicate_keys};
use bedrock_addon_core::document::Origin;
use bedrock_addon_core::load::LoadOptions;
use bedrock_addon_core::load::StackCx;
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::test_cost::{assert_linear_at, assert_linear_consuming_at};

/// The smaller input size; the larger is four times it.
const SINGLE: usize = 25_000;

/// A file of `count` entries, each a vanilla-namespaced block that asks for a shape, so that
/// the merge looks up the block's current shape.
fn shaped_entries(count: usize) -> String {
    let entries: Vec<String> = (0..count)
        .map(|n| {
            format!(r#""minecraft:b{n}": {{"textures": "t{n}", "blockshape": "cross_texture"}}"#)
        })
        .collect();
    format!(r#"{{"format_version": "1.21.40", {}}}"#, entries.join(", "))
}

#[test]
fn the_block_merge_is_linear_in_the_entries() {
    // Two packs with the same entries: every block is merged once and asks for a shape twice.
    let options = LoadOptions::default();
    let parsed = |count: usize| -> Vec<ParsedBlocks> {
        let text = shaped_entries(count);
        (0..2)
            .map(|_| ParsedBlocks {
                origin: Origin::file(
                    PackRef::app_data(),
                    PackSide::Resource,
                    PackPath::from_static("blocks.json"),
                ),
                read: resolve::read(text.as_bytes(), false),
            })
            .collect()
    };
    assert_linear_consuming_at("combine_blocks", SINGLE / 5, parsed, |files| {
        let entries = files[0].read.applied.len();
        let registry = combine_blocks(files, &StackCx::new(&options, &[]));
        assert_eq!(registry.len(), entries);
    });
}

#[test]
fn the_scan_for_repeated_keys_is_linear_in_the_keys() {
    // Every key twice.
    let text = |count: usize| {
        let entries: Vec<String> = (0..count)
            .map(|n| format!(r#""k{}": {{}}"#, n % (count / 2)))
            .collect();
        format!("{{{}}}", entries.join(", "))
    };
    assert_linear_at(
        "resolve::duplicate_keys",
        10_000,
        |count| (text(count), count / 2),
        |(text, keys)| assert_eq!(duplicate_keys(text).len(), *keys),
    );
}
