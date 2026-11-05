//! Rows r13 and r18 and validation.md rule 10: `bp.feature_rules()` walks `feature_rules/` of
//! every pack top-down, then the app data; the first rule of a `(pass, identifier)` pair wins and
//! a dropped one is `ShadowedDefinition` (info; vanilla drops it silently).

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::families::feature_rules::PlacementPass;
use bedrock_addon::id::FeatureRuleId;
use bedrock_addon::load::{AppData, LoadOptions};
use bedrock_addon::pack::{Behavior, PackStack};
use bedrock_addon::storage::PackPath;

use super::pack_common::bp;

/// A rule `minecraft:<name>` in `pass` placing `feature`, at 1.13.0.
macro_rules! rule {
    ($name:literal, $pass:literal, $feature:literal) => {
        concat!(
            r#"{"format_version": "1.13.0", "minecraft:feature_rules": {"description": {"identifier": "minecraft:"#,
            $name,
            r#"", "places_feature": ""#,
            $feature,
            r#""}, "conditions": {"placement_pass": ""#,
            $pass,
            r#""}}}"#
        )
        .as_bytes()
    };
}

const FLOWERS: &str = "overworld_after_surface_flowers_feature_rules";

fn id(name: &str) -> FeatureRuleId {
    FeatureRuleId::new(format!("minecraft:{name}")).unwrap()
}

/// The `ShadowedDefinition` findings of `rules`, with the pack name of each.
fn shadowed(
    rules: &bedrock_addon::families::feature_rules::FeatureRules,
) -> Vec<(Severity, String)> {
    rules
        .diagnostics()
        .filter(|f| matches!(f.code, DiagnosticCode::ShadowedDefinition { .. }))
        .map(|f| {
            (
                f.severity,
                f.location
                    .pack
                    .as_ref()
                    .map(|p| p.name().to_owned())
                    .unwrap_or_default(),
            )
        })
        .collect()
}

#[test]
fn r13_the_higher_pack_s_rule_of_a_pass_wins_and_the_lower_is_shadowed() {
    let top = bp(1)
        .file(
            "feature_rules/x.json",
            rule!("x", "surface_pass", "demo:high"),
        )
        .open();
    let bottom = bp(2)
        .file(
            "feature_rules/x.json",
            rule!("x", "surface_pass", "demo:low"),
        )
        .open();
    let stack = PackStack::<Behavior>::of([top, bottom], LoadOptions::default()).unwrap();
    let rules = stack.feature_rules();
    assert_eq!(rules.len(), 1);
    let rule = rules.get(PlacementPass::Surface, &id("x")).unwrap();
    assert_eq!(rule.description.places_feature.as_str(), "demo:high");
    assert_eq!(shadowed(rules), [(Severity::Info, "bp 2".to_owned())]);
    assert!(std::ptr::eq(rules, stack.feature_rules()));
}

#[test]
fn a_reference_to_feature_rules_iterates_every_rule_like_iter() {
    let top = bp(1)
        .file(
            "feature_rules/x.json",
            rule!("x", "surface_pass", "demo:high"),
        )
        .open();
    let bottom = bp(2)
        .file(
            "feature_rules/x.json",
            rule!("x", "underground_pass", "demo:low"),
        )
        .open();
    let stack = PackStack::<Behavior>::of([top, bottom], LoadOptions::default()).unwrap();
    let rules = stack.feature_rules();
    let mut seen = Vec::new();
    for (key, _) in rules {
        seen.push(key.clone());
    }
    assert_eq!(seen.len(), 2);
    assert_eq!(
        seen,
        rules.iter().map(|(key, _)| key.clone()).collect::<Vec<_>>()
    );
}

#[test]
fn r13_one_identifier_in_two_passes_is_two_rules() {
    let top = bp(1)
        .file(
            "feature_rules/x.json",
            rule!("x", "surface_pass", "demo:high"),
        )
        .open();
    let bottom = bp(2)
        .file(
            "feature_rules/x.json",
            rule!("x", "underground_pass", "demo:low"),
        )
        .open();
    let stack = PackStack::<Behavior>::of([top, bottom], LoadOptions::default()).unwrap();
    let rules = stack.feature_rules();
    assert_eq!(rules.len(), 2);
    assert!(rules.get(PlacementPass::Surface, &id("x")).is_some());
    assert!(rules.get(PlacementPass::Underground, &id("x")).is_some());
    assert!(shadowed(rules).is_empty());
}

#[test]
fn r18_an_experimental_pack_above_vanilla_overrides_a_rule_by_pass_and_identifier() {
    let experimental = bp(1)
        .file(
            "feature_rules/overworld_after_surface_flowers_feature_rules.json",
            rule!(
                "overworld_after_surface_flowers_feature_rules",
                "after_surface_pass",
                "minecraft:drop_3_flowers"
            ),
        )
        .open();
    let vanilla = bp(2)
        .file(
            "feature_rules/overworld_after_surface_flowers_feature_rules.json",
            rule!(
                "overworld_after_surface_flowers_feature_rules",
                "after_surface_pass",
                "minecraft:flowers"
            ),
        )
        .open();
    let stack = PackStack::<Behavior>::of([experimental, vanilla], LoadOptions::default()).unwrap();
    let rules = stack.feature_rules();
    let rule = rules
        .get(PlacementPass::AfterSurface, &id(FLOWERS))
        .unwrap();
    assert_eq!(
        rule.description.places_feature.as_str(),
        "minecraft:drop_3_flowers"
    );
    assert_eq!(shadowed(rules), [(Severity::Info, "bp 2".to_owned())]);
}

#[test]
fn rule_10_within_one_pack_the_first_rule_walked_wins() {
    let pack = bp(1)
        .file(
            "feature_rules/a/x.json",
            rule!("x", "surface_pass", "demo:first"),
        )
        .file(
            "feature_rules/b/x.json",
            rule!("x", "surface_pass", "demo:second"),
        )
        .open();
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default()).unwrap();
    let rules = stack.feature_rules();
    let rule = rules.get(PlacementPass::Surface, &id("x")).unwrap();
    assert_eq!(rule.description.places_feature.as_str(), "demo:first");
    assert_eq!(shadowed(rules), [(Severity::Info, "bp 1".to_owned())]);
}

#[test]
fn rules_of_an_unknown_pass_are_kept_by_their_pass_name() {
    let pack = bp(1)
        .file("feature_rules/x.json", rule!("x", "nowhere_pass", "demo:f"))
        .open();
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default()).unwrap();
    let rules = stack.feature_rules();
    assert_eq!(rules.len(), 1);
    assert!(rules.unknown_passes["nowhere_pass"].get(&id("x")).is_some());
    assert!(
        PlacementPass::ALL
            .iter()
            .all(|pass| rules.pass(*pass).is_empty())
    );
}

#[test]
fn failed_files_are_listed_and_other_files_ignored() {
    let pack = bp(1)
        .file("feature_rules/x.json", rule!("x", "surface_pass", "demo:f"))
        .file(
            "feature_rules/broken.json",
            b"{\"format_version\": \"1.13.0\", \"minecraft:feature_rules\": {}}",
        )
        .file("feature_rules/readme.txt", b"not a rule")
        .open();
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default()).unwrap();
    let rules = stack.feature_rules();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules.rejected().len(), 1);
    assert!(rules.diagnostics().any(|f| f.severity == Severity::Error));
}

#[test]
fn the_app_data_rules_come_after_every_pack() {
    let pack = bp(1)
        .file(
            "feature_rules/x.json",
            rule!("x", "surface_pass", "demo:pack"),
        )
        .open();
    let options = LoadOptions {
        app_data: Some(AppData::from_files([
            (
                PackPath::from_static("definitions/feature_rules/x.json"),
                rule!("x", "surface_pass", "demo:app_data"),
            ),
            (
                PackPath::from_static("definitions/feature_rules/y.json"),
                rule!("y", "surface_pass", "demo:app_data"),
            ),
        ])),
        ..LoadOptions::default()
    };
    let stack = PackStack::<Behavior>::of([pack], options).unwrap();
    let rules = stack.feature_rules();
    assert_eq!(rules.len(), 2);
    let x = rules.get(PlacementPass::Surface, &id("x")).unwrap();
    assert_eq!(x.description.places_feature.as_str(), "demo:pack");
    let y = rules.get(PlacementPass::Surface, &id("y")).unwrap();
    assert!(y.origin().pack.is_app_data());
}

#[test]
fn every_finding_names_its_pack_and_file() {
    // The identifier's name is not the file's stem: a kept error, the rule loads.
    let misnamed = rule!("other", "first_pass", "minecraft:f");
    let pack = bp(1)
        .file("feature_rules/misnamed.json", misnamed)
        .file("feature_rules/broken.json", b"{")
        .file("feature_rules/not_utf8.json", b"\xff")
        .open();
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default()).unwrap();
    assert_eq!(stack.feature_rules().len(), 1);
    assert_in_files(
        stack.feature_rules().diagnostics(),
        "bp 1",
        &[
            "feature_rules/misnamed.json",
            "feature_rules/broken.json",
            "feature_rules/not_utf8.json",
        ],
    );
}

/// What the stack keeps of a rule written twice in one pack, beside a lower pack's copy and a
/// file that fails: the layers of `x` in its pass under `keep_shadowed`, and every finding of
/// the group in order, each as (code, pack, path).
#[test]
fn rule_10_what_the_group_keeps_of_a_rule_written_twice_in_one_pack() {
    let top = bp(1)
        .file(
            "feature_rules/a/x.json",
            rule!("x", "surface_pass", "demo:first"),
        )
        // The stem is not the identifier's name: a kept error of the copy that loses.
        .file(
            "feature_rules/b/y.json",
            rule!("x", "surface_pass", "demo:second"),
        )
        .file("feature_rules/c/broken.json", b"{")
        .open();
    let bottom = bp(2)
        .file(
            "feature_rules/x.json",
            rule!("x", "surface_pass", "demo:low"),
        )
        .open();
    let options = LoadOptions {
        keep_shadowed: true,
        ..LoadOptions::default()
    };
    let stack = PackStack::<Behavior>::of([top, bottom], options).unwrap();
    let rules = stack.feature_rules();
    let place = |pack: &bedrock_addon::pack::PackRef, path: &PackPath| {
        format!("{} {}", pack.name(), path.as_str())
    };
    let layers: Vec<String> = rules
        .pass(PlacementPass::Surface)
        .layers(&id("x"))
        .map(|document| place(&document.origin().pack, &document.origin().path))
        .collect();
    // The copy the tie drops is kept as shadowed, beside the lower pack's.
    assert_eq!(
        layers,
        [
            "bp 1 feature_rules/a/x.json",
            "bp 1 feature_rules/b/y.json",
            "bp 2 feature_rules/x.json",
        ]
    );
    let findings: Vec<String> = rules
        .diagnostics()
        .map(|finding| {
            let at = match (&finding.location.pack, &finding.location.path) {
                (Some(pack), Some(path)) => place(pack, path),
                _ => String::new(),
            };
            format!("{} {at}", finding.code.name())
        })
        .collect();
    assert_eq!(
        findings,
        // The failed files' findings first, then the pass's registry: the dropped copy's
        // findings and its notice are the registry's now, in walk order.
        [
            "MalformedInput bp 1 feature_rules/c/broken.json",
            "InvalidValue bp 1 feature_rules/b/y.json",
            "ShadowedDefinition bp 1 feature_rules/b/y.json",
            "ShadowedDefinition bp 2 feature_rules/x.json",
        ]
    );
}

/// Every finding of `findings` names the pack `pack` and one of `files`, and each of `files`
/// has one: the loader gives each finding its document's pack and file (phase 1 step 4 pin).
fn assert_in_files<'a>(
    findings: impl IntoIterator<Item = &'a bedrock_addon::diagnostic::Diagnostic>,
    pack: &str,
    files: &[&str],
) {
    let mut named = Vec::new();
    for finding in findings {
        let in_pack = finding
            .location
            .pack
            .as_ref()
            .map(bedrock_addon::pack::PackRef::name);
        assert_eq!(in_pack, Some(pack), "{finding:?}");
        let path = finding
            .location
            .path
            .as_ref()
            .map(bedrock_addon::storage::PackPath::as_str);
        assert!(path.is_some_and(|p| files.contains(&p)), "{finding:?}");
        named.extend(path.map(str::to_owned));
    }
    for file in files {
        assert!(named.iter().any(|p| p == file), "no finding names {file}");
    }
}
