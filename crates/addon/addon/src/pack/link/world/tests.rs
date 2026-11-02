//! How the cost of the group's rules grows: each rule that does more than one lookup per
//! reference, and each family checker the group calls, on `n` references against `4 n`.

#![allow(
    unused_imports,
    dead_code,
    reason = "which rules compile depends on the features"
)]

use bedrock_addon_core::load::LoadOptions;

use super::super::tests::support::{File, pack, set, set_of};
use super::super::{LinkOptions, link_with_rules};
use crate::pack::StackSet;
use crate::test_cost::assert_linear_at;

/// A path for the life of the test process.
fn leaked(path: String) -> &'static str {
    Box::leak(path.into_boxed_str())
}

/// A world of one behaviour pack with `files`.
fn world(files: &[(&'static str, String)]) -> StackSet {
    let files: Vec<File<'_>> = files
        .iter()
        .map(|(path, text)| (*path, Some(text.as_str())))
        .collect();
    set_of(
        vec![pack(1, "data", &files)],
        Vec::new(),
        &LoadOptions::default(),
    )
}

/// `n` subtree nodes that name no tree, and two trees that name each other `n` times.
#[cfg(feature = "behavior_tree")]
#[test]
fn the_subtree_rule_is_linear_in_the_nodes_and_in_the_edges() {
    fn tree(name: &str, target: &str, nodes: usize) -> String {
        let node = format!(r#"{{"minecraft:subtree": {{"subtree_id": "{target}"}}}}"#);
        let children = vec![node; nodes].join(",");
        format!(
            r#"{{"minecraft:behavior_tree": {{"minecraft:name": "{name}", "minecraft:root": {{"minecraft:selector": {{"children": [{children}]}}}}}}}}"#
        )
    }
    let run = |set: &StackSet, expected: usize| {
        let found = link_with_rules(set, LinkOptions::default(), &[super::behavior_tree::run]);
        assert_eq!(found.len(), expected);
    };
    assert_linear_at(
        "link: n subtree nodes that name no tree",
        20_000,
        |n| {
            let set = world(&[("behavior_trees/wide.json", tree("wide", "nowhere", n))]);
            // The load is not the rule's.
            assert!(set.behavior.behavior_trees().len() > 1);
            (set, n)
        },
        |(set, n)| run(set, *n),
    );
    assert_linear_at(
        "link: two trees that name each other n times",
        10_000,
        |n| {
            let set = world(&[
                ("behavior_trees/a.json", tree("a", "b", n)),
                ("behavior_trees/b.json", tree("b", "a", n)),
            ]);
            assert!(set.behavior.behavior_trees().len() > 2);
            (set, 2 * n)
        },
        |(set, n)| run(set, *n),
    );
}

/// `n` category files, each with a name nothing defines; and one old preset whose
/// `exclusion_list` has `n` entries that are neither a block nor an entity.
#[cfg(all(feature = "aim_assist", feature = "block", feature = "entity"))]
#[test]
fn the_aim_assist_rule_is_linear_in_the_files_and_in_the_entries() {
    let run = |set: &StackSet, expected: usize| {
        let found = link_with_rules(set, LinkOptions::default(), &[super::aim_assist::run]);
        assert_eq!(found.len(), expected);
    };
    assert_linear_at(
        "link: n category files",
        4_000,
        |n| {
            let files: Vec<(&'static str, String)> = (0..n)
                .map(|i| {
                    (
                        leaked(format!("aim_assist/categories/c{i}.json")),
                        format!(
                            r#"{{"format_version": "1.21.50", "minecraft:aim_assist_categories": {{"categories": [{{"name": "demo:c{i}", "priorities": {{"entities": {{"demo:ghost": 1}}}}}}]}}}}"#
                        ),
                    )
                })
                .collect();
            let set = world(&files);
            assert_eq!(set.behavior.aim_assist().categories.len(), n);
            (set, n)
        },
        |(set, n)| run(set, *n),
    );
    assert_linear_at(
        "link: n entries of an old exclusion list",
        20_000,
        |n| {
            let entries: Vec<String> = (0..n).map(|i| format!(r#""demo:neither_{i}""#)).collect();
            let preset = format!(
                r#"{{"format_version": "1.21.50", "minecraft:aim_assist_preset": {{"identifier": "demo:aim", "exclusion_list": [{}]}}}}"#,
                entries.join(",")
            );
            let set = world(&[("aim_assist/presets/p.json", preset)]);
            assert_eq!(set.behavior.aim_assist().presets.len(), 1);
            (set, n)
        },
        |(set, n)| run(set, *n),
    );
}

/// `n` rules of the pregeneration pass, each placing a feature of its own that is
/// no cave carver (the feature, its document and the rule's pack are looked up per rule).
#[cfg(all(feature = "feature_rules", feature = "feature"))]
#[test]
fn the_feature_rule_rule_is_linear_in_the_rules() {
    assert_linear_at(
        "link: n rules placing n features",
        4_000,
        |n| {
            let mut files: Vec<(&'static str, String)> = Vec::with_capacity(2 * n);
            for i in 0..n {
                files.push((
                    leaked(format!("feature_rules/r{i}.json")),
                    format!(
                        r#"{{"format_version": "1.13.0", "minecraft:feature_rules": {{"description": {{"identifier": "demo:r{i}", "places_feature": "demo:f{i}"}}, "conditions": {{"placement_pass": "pregeneration_pass"}}}}}}"#
                    ),
                ));
                files.push((
                    leaked(format!("features/f{i}.json")),
                    format!(
                        r#"{{"format_version": "1.13.0", "minecraft:ore_feature": {{"description": {{"identifier": "demo:f{i}"}}, "count": 8}}}}"#
                    ),
                ));
            }
            let set = world(&files);
            assert_eq!(set.behavior.features().len(), n);
            assert_eq!(set.behavior.feature_rules().len(), n);
            (set, n)
        },
        |(set, n)| {
            let found = link_with_rules(set, LinkOptions::default(), &[super::feature_rules::run]);
            assert_eq!(found.len(), *n);
        },
    );
}

/// One biome replacing `n` biomes a pack of the world adds (the biome, the game's
/// list and the pack of the definition are looked up per target).
#[cfg(feature = "biome")]
#[test]
fn the_replacement_rule_is_linear_in_the_targets() {
    assert_linear_at(
        "link: n targets among n biomes",
        6_000,
        |n| {
            let biome = |id: &str, components: &str| {
                format!(
                    r#"{{"format_version": "1.26.30", "minecraft:biome": {{"description": {{"identifier": "{id}"}}, "components": {{{components}}}}}}}"#
                )
            };
            let mut files: Vec<(&'static str, String)> = (0..n)
                .map(|i| {
                    (
                        leaked(format!("biomes/b{i}.json")),
                        biome(&format!("demo:b{i}"), ""),
                    )
                })
                .collect();
            let targets: Vec<String> = (0..n).map(|i| format!(r#""demo:b{i}""#)).collect();
            files.push((
                "biomes/replacer.json",
                biome(
                    "demo:replacer",
                    &format!(
                        r#""minecraft:replace_biomes": {{"replacements": [{{"dimension": "minecraft:overworld", "targets": [{}], "amount": 0.5, "noise_frequency_scale": 1.0}}]}}"#,
                        targets.join(",")
                    ),
                ),
            ));
            let set = world(&files);
            assert_eq!(set.behavior.biomes().len(), n + 1);
            (set, n)
        },
        |(set, n)| {
            let found = link_with_rules(set, LinkOptions::default(), &[super::biome::run]);
            assert_eq!(found.len(), *n);
        },
    );
}
