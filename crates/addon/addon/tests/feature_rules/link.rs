//! Rows r15 and r16 and rules 8, 9 and 12: the checks of a rule's
//! `places_feature` against the loaded features run in the link pass:
//! the tests run through `StackSet::link`.

use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackOrigin, PackRepository, PackSelection, WorldPacks};

use super::pack_common::{Spec, bp};

/// A rule `demo:r` in `pass` placing `feature`.
fn rule(pass: &str, feature: &str) -> &'static [u8] {
    format!(
        r#"{{"format_version": "1.13.0", "minecraft:feature_rules": {{"description": {{"identifier": "demo:r", "places_feature": "{feature}"}}, "conditions": {{"placement_pass": "{pass}"}}}}}}"#
    )
    .leak()
    .as_bytes()
}

/// A feature document `identifier` of type `payload` (its whole payload member text).
fn feature(payload: &str, identifier: &str, body: &str) -> &'static [u8] {
    format!(
        r#"{{"format_version": "1.13.0", "{payload}": {{"description": {{"identifier": "{identifier}"}}{body}}}}}"#
    )
    .leak()
    .as_bytes()
}

/// The link findings of a world with the behaviour pack `spec` (of `origin`).
fn link(spec: &Spec, origin: PackOrigin) -> Diagnostics {
    link_all(&[(spec, origin)])
}

/// The link findings of a world with the behaviour packs `specs`, top first, each of its
/// origin.
fn link_all(specs: &[(&Spec, PackOrigin)]) -> Diagnostics {
    let mut repository = PackRepository::new();
    let mut behavior = Vec::new();
    for (spec, origin) in specs {
        repository.add_pack(spec.open(), *origin);
        let id = spec.id();
        behavior.push(PackSelection {
            pack_id: id.uuid,
            version: id.version,
            subpack: None,
        });
    }
    let world = WorldPacks {
        behavior,
        resource: Vec::new(),
    };
    repository.stacks(&world, LoadOptions::default()).link()
}

/// Whether `d` has a finding with text `text` at `severity`.
fn has(d: &Diagnostics, severity: Severity, text: &str) -> bool {
    d.iter()
        .any(|f| f.severity == severity && f.code.to_string() == text)
}

#[test]
fn r15_pregeneration_pass_places_only_cave_carvers() {
    let text = "\"cave_carver_feature\" is the only valid feature in \"pregeneration_pass\" placement pass.";
    let scatter = bp(1)
        .file(
            "feature_rules/r.json",
            rule("pregeneration_pass", "demo:scatter"),
        )
        .file(
            "features/scatter.json",
            feature(
                "minecraft:scatter_feature",
                "demo:scatter",
                // the root; with `distribution` the file was rejected and the rule placed nothing)
                r#", "places_feature": "demo:other", "iterations": 1, "x": 0, "y": 0, "z": 0"#,
            ),
        );
    let d = link(&scatter, PackOrigin::World);
    assert!(has(&d, Severity::Error, text), "{d:#?}");
    assert!(
        d.iter()
            .any(|f| f.code.id() == "core/invalid_reference_worded")
    );
    let carver = bp(2)
        .file(
            "feature_rules/r.json",
            rule("pregeneration_pass", "demo:carver"),
        )
        .file(
            "features/carver.json",
            feature("minecraft:cave_carver_feature", "demo:carver", ""),
        );
    assert!(!has(
        &link(&carver, PackOrigin::World),
        Severity::Error,
        text
    ));
}

// of that type in a pack of the game, not a feature of that name; the fixture has one)
#[test]
fn r16_a_non_base_game_rule_cannot_place_an_internal_vanilla_feature() {
    let text = "Feature rule demo:r can't place internal feature type.";
    // A feature of an internal type, as only a pack of the game can define one.
    let layout = feature(
        "minecraft:rect_layout",
        "minecraft:layout",
        r#", "feature_areas": [{"feature": "minecraft:layout", "area_dimensions": [1, 1]}]"#,
    );
    let game = bp(2).file("features/layout.json", layout);
    let world = bp(1).file(
        "feature_rules/r.json",
        rule("surface_pass", "minecraft:layout"),
    );
    let d = link_all(&[(&world, PackOrigin::World), (&game, PackOrigin::Package)]);
    assert!(has(&d, Severity::Error, text), "{d:#?}");
    assert!(
        d.iter()
            .any(|f| f.code.id() == "core/invalid_reference_worded")
    );
    // The same rule in the game's own pack.
    let base_game = bp(2).file("features/layout.json", layout).file(
        "feature_rules/r.json",
        rule("surface_pass", "minecraft:layout"),
    );
    assert!(!has(
        &link(&base_game, PackOrigin::Package),
        Severity::Error,
        text
    ));
}

#[test]
fn rule_9_a_places_feature_no_feature_defines_is_an_info() {
    let spec = bp(1).file("feature_rules/r.json", rule("surface_pass", "demo:nowhere"));
    let d = link(&spec, PackOrigin::World);
    assert!(
        d.iter()
            .any(|f| f.severity == Severity::Info && f.code.id() == "core/unresolved_reference"),
        "{d:#?}"
    );
    assert!(!d.iter().any(|f| f.severity > Severity::Info));
}
