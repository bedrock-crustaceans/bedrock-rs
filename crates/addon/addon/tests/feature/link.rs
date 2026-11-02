//! Rules that need another family's definitions and run in the link pass, through
//! `StackSet::link`.

#![cfg(feature = "block")]

use bedrock_addon::diagnostic::{DiagnosticCode, Diagnostics, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackOrigin, PackRepository, PackSelection, StackSet, WorldPacks};

use super::common::fixture;
use super::pack_common::{Spec, bp};

/// A stack of one behaviour pack built from `spec`, with Upcoming Creator Features (the
/// `minecraft:multi_block` trait needs it).
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
    let options = LoadOptions {
        upcoming_creator_features: true,
        ..LoadOptions::default()
    };
    repository.stacks(&world, options)
}

/// Whether `d` has a finding of `severity` whose code is `name` and whose message is `text`,
/// at the file `path`.
fn has(d: &Diagnostics, name: &str, severity: Severity, text: &str, path: &str) -> bool {
    d.iter().any(|f| {
        f.code.name() == name
            && f.severity == severity
            && f.code.to_string() == text
            && f.location.path.as_ref().is_some_and(|p| p.as_str() == path)
    })
}

/// Leaks a fixture's text: `Spec::file` takes `'static` bytes.
fn leaked(name: &str) -> &'static [u8] {
    Box::leak(fixture(name).into_bytes().into_boxed_slice())
}

#[test]
fn f19_multi_block_places_block_must_be_a_starting_multi_block_part() {
    let spec = bp(1)
        .file("blocks/lamp.json", leaked("f19_lamp_block.json"))
        .file("blocks/door.json", leaked("f19_door_block.json"))
        .file(
            "features/place_lamp.json",
            leaked("f19_multi_block_features.json"),
        )
        .file(
            "features/place_door_top.json",
            leaked("f19_multi_block_part_feature.json"),
        );
    let set = stacks(&spec);
    // Both documents load: the rules need the block definitions.
    assert_eq!(set.behavior.features().len(), 2);
    let d = set.link();
    // `InvalidReference`, error (the game's `InvalidValue` is
    // one code per kind of miss behind).
    assert!(
        has(
            &d,
            "InvalidReference",
            Severity::Error,
            "Must place a multi-block in a 'minecraft:multi_block_feature'",
            "features/place_lamp.json",
        ),
        "{d:#?}"
    );
    // A part other than the starting one (`minecraft:multi_block_part` 1).
    assert!(
        has(
            &d,
            "InvalidReference",
            Severity::Error,
            "If trying to place a multi-block in a 'minecraft:multi_block_feature' only the starting part may be used in field 'places_block'",
            "features/place_door_top.json",
        ),
        "{d:#?}"
    );
    assert!(
        !d.iter()
            .any(|f| matches!(f.code, DiagnosticCode::InvalidValue { .. })
                && f.location
                    .path
                    .as_ref()
                    .is_some_and(|p| p.as_str().contains("door.json"))),
        "the door block itself is valid"
    );
}

#[test]
fn w4_single_block_places_block_must_resolve() {
    let spec = bp(1)
        .file("blocks/lamp.json", leaked("f19_lamp_block.json"))
        .file(
            "features/place_missing.json",
            leaked("w4_single_block_unresolved.json"),
        );
    let set = stacks(&spec);
    // The feature loads: the setter's message does not fail the file.
    assert_eq!(set.behavior.features().len(), 1);
    let d = set.link();
    assert!(
        has(
            &d,
            "UnresolvedReference",
            Severity::Error,
            "Failed to resolve block for 'places_block' property.",
            "features/place_missing.json",
        ),
        "{d:#?}"
    );
}
