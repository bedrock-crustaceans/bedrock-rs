//! 20 W8 (the feature a rule places), W9 (the pregeneration pass places cave carvers only) and
//! W10 (a rule of a pack that is not the game's cannot place a feature of an internal type).
//! W11 (the biome filter) is audited in `audit.rs`.

#![allow(unused_imports, reason = "which rows compile depends on the features")]

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::LinkOptions;

use super::super::common::{hosted, link, link_with};
use super::feature::{feature, vein};
use super::{finding, said, stacks_with_game};

const RULE: &str = "feature_rules/r.json";
const AT: &str = "minecraft:feature_rules/description/places_feature";

/// A rule `demo:r` in `pass` placing `feature`.
fn rule(pass: &str, feature: &str) -> String {
    format!(
        r#"{{"format_version": "1.13.0", "minecraft:feature_rules": {{"description": {{"identifier": "demo:r", "places_feature": "{feature}"}}, "conditions": {{"placement_pass": "{pass}"}}}}}}"#
    )
}

// ---- W8 --------------------------------------------------------------------------------------

#[test]
fn w8_a_feature_a_pack_defines_or_a_host_has_resolves() {
    let (text, vein) = (rule("surface_pass", "demo:vein"), vein("demo:vein"));
    let found = link(&[(RULE, text.as_str()), ("features/vein.json", &vein)], &[]);
    assert_eq!(said(&found, &["feature"]), []);
    let text = rule("surface_pass", "minecraft:oak_tree_feature");
    let found = link_with(
        &[(RULE, text.as_str())],
        &[],
        hosted(&[("feature", "minecraft:oak_tree_feature")]),
        LinkOptions::default(),
    );
    assert_eq!(said(&found, &["feature"]), []);
}

/// K112: a feature the game registers in code (no file defines it; the game's own rules
/// place it) is the game's without a host; a feature of the game's behaviour pack is the
/// pack's or a host's to answer for.
#[test]
fn w8_a_feature_the_game_registers_in_code_resolves_without_a_host() {
    for name in [
        "minecraft:legacy:forest_tree_feature",
        "minecraft:kelp_feature",
    ] {
        let text = rule("surface_pass", name);
        let found = link(&[(RULE, text.as_str())], &[]);
        assert_eq!(said(&found, &["feature"]), [], "{name}");
    }
    let text = rule("surface_pass", "minecraft:oak_tree_feature");
    let found = link(&[(RULE, text.as_str())], &[]);
    assert_eq!(
        said(&found, &["feature"]),
        [finding(
            "core/unresolved_reference",
            Severity::Info,
            RULE,
            AT,
            "feature",
            "minecraft:oak_tree_feature"
        )]
    );
}

#[test]
fn w8_a_feature_nothing_defines_is_an_information_and_a_lint() {
    let text = rule("surface_pass", "demo:gone");
    let files = [(RULE, text.as_str())];
    let found = link(&files, &[]);
    assert_eq!(
        said(&found, &["feature"]),
        [finding(
            "core/unresolved_reference",
            Severity::Info,
            RULE,
            AT,
            "feature",
            "demo:gone"
        )]
    );
    // The game reserves the name and says nothing: a lint, left out with the lints.
    let found = link_with(
        &files,
        &[],
        LoadOptions::default(),
        LinkOptions {
            lints: false,
            ..Default::default()
        },
    );
    assert_eq!(said(&found, &["feature"]), []);
}

// ---- W9 --------------------------------------------------------------------------------------

#[test]
fn w9_the_pregeneration_pass_places_cave_carvers_only() {
    const TEXT: &str = "\"cave_carver_feature\" is the only valid feature in \"pregeneration_pass\" placement pass.";
    let found = |pass: &str, kind: &str, members: &str| {
        let (text, placed) = (
            rule(pass, "demo:placed"),
            feature("1.13.0", kind, "demo:placed", members),
        );
        let found = link(
            &[(RULE, text.as_str()), ("features/placed.json", &placed)],
            &[],
        );
        let said = said(&found, &["feature"]);
        assert!(
            super::texts(&found, "feature")
                .iter()
                .all(|text| text == TEXT),
            "{found:#?}"
        );
        // The game logs at level 3 and returns false, the rule
        // kept: a kept error.
        assert!(
            found
                .iter()
                .filter(|finding| finding.code.id() == "core/invalid_reference_worded")
                .all(|finding| finding.kept),
            "{found:#?}"
        );
        said
    };
    let invalid = [finding(
        "core/invalid_reference_worded",
        Severity::Error,
        RULE,
        AT,
        "feature",
        "demo:placed",
    )];
    assert_eq!(
        found("pregeneration_pass", "ore_feature", r#", "count": 8"#),
        invalid
    );
    // The three carvers are cave carvers.
    for carver in [
        "cave_carver_feature",
        "nether_cave_carver_feature",
        "underwater_cave_carver_feature",
    ] {
        assert_eq!(found("pregeneration_pass", carver, ""), [], "{carver}");
    }
    // Any other pass places any feature.
    assert_eq!(found("first_pass", "ore_feature", r#", "count": 8"#), []);
    // A feature nothing defines is W8's, not this rule's.
    let text = rule("pregeneration_pass", "demo:gone");
    let found = link(&[(RULE, text.as_str())], &[]);
    assert_eq!(
        said(&found, &["feature"])
            .iter()
            .map(|said| said.0)
            .collect::<Vec<_>>(),
        ["core/unresolved_reference"]
    );
}

// ---- W10 -------------------------------------------------------------------------------------

#[test]
fn w10_a_rule_outside_the_games_packs_cannot_place_a_feature_of_an_internal_type() {
    const TEXT: &str = "Feature rule demo:r can't place internal feature type.";
    let layout = feature(
        "1.13.0",
        "rect_layout",
        "minecraft:layout",
        r#", "feature_areas": [{"feature": "minecraft:vein", "area_dimensions": [1, 1]}]"#,
    );
    let (vein, text) = (
        vein("minecraft:vein"),
        rule("surface_pass", "minecraft:layout"),
    );
    let game = [
        ("features/layout.json", layout.as_str()),
        ("features/vein.json", vein.as_str()),
    ];
    // The rule in a pack of the world.
    let set = stacks_with_game(&[(RULE, text.as_str())], &game, LoadOptions::default());
    assert_eq!(set.behavior.features().len(), 2);
    let found = set.link();
    assert_eq!(
        said(&found, &["feature"]),
        [finding(
            "core/invalid_reference_worded",
            Severity::Error,
            RULE,
            AT,
            "feature",
            "minecraft:layout"
        )]
    );
    assert_eq!(super::texts(&found, "feature"), [TEXT]);
    // The game logs at level 3 and skips the rule,
    // which stays registered: a kept error.
    assert!(
        found
            .iter()
            .filter(|finding| finding.code.id() == "core/invalid_reference_worded")
            .all(|finding| finding.kept),
        "{found:#?}"
    );
    // The same rule in the game's own pack, and a rule of the world placing a feature of
    // another type.
    let mut with_rule = game.to_vec();
    with_rule.push((RULE, text.as_str()));
    let set = stacks_with_game(&[], &with_rule, LoadOptions::default());
    assert_eq!(said(&set.link(), &["feature"]), []);
    let text = rule("surface_pass", "minecraft:vein");
    let set = stacks_with_game(&[(RULE, text.as_str())], &game, LoadOptions::default());
    assert_eq!(said(&set.link(), &["feature"]), []);
}
