//! 20 W25: the blocks of a structure's palettes. W26 (its entities) is audited in `audit.rs`.

#![allow(unused_imports, reason = "which rows compile depends on the features")]

use bedrock_addon::families::structure::{Palette, Structure};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{LinkOptions, StackSet};
use nbtx::{Compound, Value};

use super::super::common::{behavior_pack_of, hosted, leak, stacks_of};
#[cfg(feature = "block")]
use super::{said, unresolved_info};

const HUT: &str = "structures/demo/hut.mcstructure";

/// A palette entry as the game saves one: `{name, states, version}`.
fn entry(name: &str) -> Compound {
    Compound::from_iter([
        ("name".into(), Value::String(name.into())),
        ("states".into(), Value::Compound(Compound::new())),
        ("version".into(), Value::Int(18_168_865)),
    ])
}

/// The bytes of a one-block structure whose palettes are `palettes`: each a name and the
/// names of its blocks.
fn structure(palettes: &[(&str, &[&str])]) -> &'static [u8] {
    let mut structure = Structure::new([1, 1, 1]);
    structure.structure.palette = palettes
        .iter()
        .map(|(name, blocks)| {
            let palette = Palette {
                block_palette: blocks.iter().map(|block| entry(block)).collect(),
                ..Palette::default()
            };
            ((*name).to_owned(), palette)
        })
        .collect();
    Box::leak(
        structure
            .to_bytes()
            .expect("a structure encodes")
            .into_boxed_slice(),
    )
}

/// A world with the structure at [`HUT`] and the text files `more`.
fn world(bytes: &'static [u8], more: &[(&'static str, &str)], options: LoadOptions) -> StackSet {
    let mut files = vec![(HUT, bytes)];
    files.extend(more.iter().map(|(path, text)| (*path, leak(text))));
    let set = stacks_of(vec![behavior_pack_of(1, &files)], Vec::new(), options);
    assert_eq!(set.behavior.structures().iter().count(), 1);
    set
}

#[cfg(feature = "block")]
#[test]
fn w25_a_palette_block_a_pack_defines_or_the_game_has_resolves() {
    let lamp = super::block("demo:lamp");
    let bytes = structure(&[(
        "default",
        &["minecraft:stone", "demo:lamp", "minecraft:air"],
    )]);
    let set = world(
        bytes,
        &[("blocks/lamp.json", lamp.as_str())],
        LoadOptions::default(),
    );
    assert_eq!(said(&set.link(), &["block"]), []);
    let bytes = structure(&[("default", &["demo:lamp"])]);
    let set = world(bytes, &[], hosted(&[("block", "demo:lamp")]));
    assert_eq!(said(&set.link(), &["block"]), []);
}

#[cfg(feature = "block")]
#[test]
fn w25_a_palette_block_nothing_defines_is_an_information_at_its_name_in_every_palette() {
    let bytes = structure(&[
        ("default", &["minecraft:stone", "demo:lamp"]),
        ("mossy", &["demo:moss", "minecraft:stone"]),
    ]);
    let set = world(bytes, &[], LoadOptions::default());
    assert_eq!(
        said(&set.link(), &["block"]),
        [
            unresolved_info(
                HUT,
                "structure/palette/default/block_palette/1/name",
                "block",
                "demo:lamp"
            ),
            unresolved_info(
                HUT,
                "structure/palette/mossy/block_palette/0/name",
                "block",
                "demo:moss"
            ),
        ]
    );
    // The game puts the placeholder there in silence (`tryGetBlock`): an information, a lint.
    assert_eq!(
        said(
            &set.link_with(&LinkOptions {
                lints: false,
                ..Default::default()
            }),
            &["block"]
        ),
        []
    );
}

/// Without the `block` module nobody knows a pack's blocks.
#[cfg(not(feature = "block"))]
#[test]
fn w25_without_the_block_module_no_palette_block_is_reported() {
    let bytes = structure(&[("default", &["demo:lamp"])]);
    let set = world(bytes, &[], hosted(&[]));
    assert!(set.link().is_empty());
}
