//! The structure rows of the link pass, through `StackSet::link`.

// (each rule of this file relates its module and `block`)
#![cfg(feature = "block")]

use bedrock_addon::diagnostic::{DiagnosticDetail, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackOrigin, PackRepository, PackSelection, StackSet, WorldPacks};

use super::common::fixture;
use super::pack_common::{Spec, bp};

/// A stack set of one behaviour pack built from `spec`.
fn stacks(spec: &Spec) -> StackSet {
    let id = spec.id();
    let mut repository = PackRepository::new();
    repository.add_pack(spec.open(), PackOrigin::World);
    let world = WorldPacks {
        behavior: vec![PackSelection {
            pack_id: id.uuid,
            version: id.version,
            subpack: None,
        }],
        resource: Vec::new(),
    };
    repository.stacks(&world, LoadOptions::default())
}

#[test]
fn s32_an_unknown_palette_block_is_an_unresolved_reference_information() {
    let bytes: &'static [u8] = Box::leak(fixture("s32_unknown_block").into_boxed_slice());
    let stone: &'static [u8] = Box::leak(fixture("s01_vanilla_1x1x1").into_boxed_slice());
    let spec = bp(1)
        .file("structures/demo/unknown.mcstructure", bytes)
        .file("structures/demo/stone.mcstructure", stone);
    let set = stacks(&spec);
    // Both structures load: the name is checked in the link pass.
    assert_eq!(set.behavior.structures().iter().count(), 2);
    let d = set.link();
    let unresolved: Vec<_> = d
        .iter()
        .filter(|f| f.code.name() == "UnresolvedReference")
        .collect();
    // Vanilla puts an unknown-block placeholder in its place in silence: an information (20
    // W25), at the palette entry; `minecraft:stone` is a built-in and resolves.
    assert_eq!(unresolved.len(), 1, "{d:#?}");
    let finding = unresolved[0];
    assert_eq!(finding.severity, Severity::Info);
    assert_eq!(
        finding.location.path.as_ref().unwrap().as_str(),
        "structures/demo/unknown.mcstructure"
    );
    assert_eq!(
        finding.location.json_path.to_string(),
        "structure/palette/default/block_palette/0/name"
    );
    assert!(matches!(
        finding.detail.as_deref(),
        Some(DiagnosticDetail::Reference { kind: "block", id, .. }) if id == "demo:missing"
    ));
}
