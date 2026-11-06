//! Characterization of the severity of an unknown member or component (phase 7, grading):
//! what each format version tells of the same small documents, through both of the reader's
//! paths. A document at 1.26.50, 1.26.60 or beta is read by the newest struct's reader; one at
//! 1.21.88, 1.21.89 or 1.26.20 is read by an older struct and carried to the newest by the
//! steps (`into_latest`). The tables pin what the reader tells today, whatever mechanism grades
//! it, so that moving the grading from the post-parse lifts to the raise sites changes nothing.

use std::fmt::Write as _;

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::document::Content;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_entity::Entity;

use super::common::{cx, doc};

/// The versions, oldest first: the last unstrict slice, the first strict one, the
/// `entity_version` slice, the newest struct's own slice and the one before, and beta.
const VERSIONS: [&str; 6] = [
    "1.21.88", "1.21.89", "1.26.20", "1.26.50", "1.26.60", "beta",
];

/// What each version tells of `payload`: `version loaded|refused` then one line per finding
/// at information or above, `severity code path`. The beta document is read with the Beta APIs
/// toggle on, which its envelope needs.
fn told(payload: &str) -> String {
    let mut out = String::new();
    for version in VERSIONS {
        let mut context = cx();
        context.beta_apis = version == "beta";
        let path = PackPath::new("entities/test.json").unwrap();
        let text = doc(version, payload);
        let (found, loaded) = match Entity::parse(&path, text.as_bytes(), &context) {
            Ok(document) => (document.diagnostics().clone(), true),
            Err(rejected) => (rejected.diagnostics, false),
        };
        out.push_str(version);
        out.push_str(if loaded { " loaded\n" } else { " refused\n" });
        for finding in found.iter().filter(|f| f.severity >= Severity::Info) {
            let _ = writeln!(
                out,
                "  {:?} {} {}",
                finding.severity,
                finding.code.name(),
                finding.location.json_path
            );
        }
    }
    out
}

#[test]
fn an_unknown_root_member_is_graded_by_the_version() {
    let found =
        told(r#"{"description": {"identifier": "demo:x"}, "components": {}, "made_up": {}}"#);
    assert_eq!(
        found,
        r"1.21.88 loaded
  Warning UnrecognizedField minecraft:entity/made_up
1.21.89 refused
  Error UnrecognizedField minecraft:entity/made_up
  Error DocumentRejected 
1.26.20 refused
  Error UnrecognizedField minecraft:entity/made_up
  Error DocumentRejected 
1.26.50 refused
  Error UnrecognizedField minecraft:entity/made_up
  Error DocumentRejected 
1.26.60 refused
  Error UnrecognizedField minecraft:entity/made_up
  Error DocumentRejected 
beta refused
  Error UnrecognizedField minecraft:entity/made_up
  Error DocumentRejected 
",
        "\n{found}"
    );
}

#[test]
fn an_unknown_description_member_is_graded_by_the_version() {
    let found =
        told(r#"{"description": {"identifier": "demo:x", "demo_note": 1}, "components": {}}"#);
    assert_eq!(
        found,
        r"1.21.88 loaded
  Warning UnrecognizedField minecraft:entity/description/demo_note
1.21.89 refused
  Error UnrecognizedField minecraft:entity/description/demo_note
  Error DocumentRejected 
1.26.20 refused
  Error UnrecognizedField minecraft:entity/description/demo_note
  Error DocumentRejected 
1.26.50 refused
  Error UnrecognizedField minecraft:entity/description/demo_note
  Error DocumentRejected 
1.26.60 refused
  Error UnrecognizedField minecraft:entity/description/demo_note
  Error DocumentRejected 
beta refused
  Error UnrecognizedField minecraft:entity/description/demo_note
  Error DocumentRejected 
",
        "\n{found}"
    );
}

#[test]
fn an_unknown_component_is_graded_by_the_version() {
    let found =
        told(r#"{"description": {"identifier": "demo:x"}, "components": {"demo:made_up": {}}}"#);
    assert_eq!(
        found,
        r"1.21.88 loaded
  Warning UnrecognizedComponent minecraft:entity/components/demo:made_up
1.21.89 refused
  Error UnrecognizedComponent minecraft:entity/components/demo:made_up
  Error DocumentRejected 
1.26.20 refused
  Error UnrecognizedComponent minecraft:entity/components/demo:made_up
  Error DocumentRejected 
1.26.50 refused
  Error UnrecognizedComponent minecraft:entity/components/demo:made_up
  Error DocumentRejected 
1.26.60 refused
  Error UnrecognizedComponent minecraft:entity/components/demo:made_up
  Error DocumentRejected 
beta refused
  Error UnrecognizedComponent minecraft:entity/components/demo:made_up
  Error DocumentRejected 
",
        "\n{found}"
    );
}

#[test]
fn an_unknown_member_of_a_typed_component_is_graded_by_the_version() {
    let found = told(
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:pushable_by_block": {"unexpected_field": 1}}}"#,
    );
    assert_eq!(
        found,
        r"1.21.88 loaded
  Error UnrecognizedField minecraft:entity/components/minecraft:pushable_by_block/unexpected_field
  Info ValueRewritten minecraft:entity/components/minecraft:pushable_by_block
  Info DroppedField minecraft:entity/components/minecraft:pushable_by_block/unexpected_field
1.21.89 loaded
  Error UnrecognizedField minecraft:entity/components/minecraft:pushable_by_block/unexpected_field
  Info ValueRewritten minecraft:entity/components/minecraft:pushable_by_block
  Info DroppedField minecraft:entity/components/minecraft:pushable_by_block/unexpected_field
1.26.20 loaded
  Error UnrecognizedField minecraft:entity/components/minecraft:pushable_by_block/unexpected_field
  Info ValueRewritten minecraft:entity/components/minecraft:pushable_by_block
  Info DroppedField minecraft:entity/components/minecraft:pushable_by_block/unexpected_field
1.26.50 loaded
  Error UnrecognizedField minecraft:entity/components/minecraft:pushable_by_block/unexpected_field
  Info ValueRewritten minecraft:entity/components/minecraft:pushable_by_block
  Info DroppedField minecraft:entity/components/minecraft:pushable_by_block/unexpected_field
1.26.60 refused
  Error UnrecognizedField minecraft:entity/components/minecraft:pushable_by_block/unexpected_field
  Error DocumentRejected 
beta refused
  Error UnrecognizedField minecraft:entity/components/minecraft:pushable_by_block/unexpected_field
  Error DocumentRejected 
",
        "\n{found}"
    );
}

#[test]
fn an_unknown_member_of_a_goal_is_graded_by_the_version() {
    let found = told(
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:behavior.float": {"priority": 1, "plain": 1}}}"#,
    );
    assert_eq!(
        found,
        r"1.21.88 loaded
  Info UnrecognizedField minecraft:entity/components/minecraft:behavior.float/plain
1.21.89 loaded
  Info UnrecognizedField minecraft:entity/components/minecraft:behavior.float/plain
1.26.20 refused
  Error UnrecognizedField minecraft:entity/components/minecraft:behavior.float/plain
  Error DocumentRejected 
1.26.50 refused
  Error UnrecognizedField minecraft:entity/components/minecraft:behavior.float/plain
  Error DocumentRejected 
1.26.60 refused
  Error UnrecognizedField minecraft:entity/components/minecraft:behavior.float/plain
  Error DocumentRejected 
beta refused
  Error UnrecognizedField minecraft:entity/components/minecraft:behavior.float/plain
  Error DocumentRejected 
",
        "\n{found}"
    );
}

#[test]
fn an_unknown_member_of_a_component_group_is_graded_by_the_version() {
    let found = told(
        r#"{"description": {"identifier": "demo:x"}, "components": {}, "component_groups": {"g": {"demo:made_up": {}, "minecraft:pushable_by_block": {"unexpected_field": 1}}}}"#,
    );
    assert_eq!(
        found,
        r"1.21.88 loaded
  Warning UnrecognizedComponent minecraft:entity/component_groups/g/demo:made_up
  Error UnrecognizedField minecraft:entity/component_groups/g/minecraft:pushable_by_block/unexpected_field
  Info ValueRewritten minecraft:entity/component_groups/g/minecraft:pushable_by_block
  Info DroppedField minecraft:entity/component_groups/g/minecraft:pushable_by_block/unexpected_field
1.21.89 refused
  Error UnrecognizedComponent minecraft:entity/component_groups/g/demo:made_up
  Error UnrecognizedField minecraft:entity/component_groups/g/minecraft:pushable_by_block/unexpected_field
  Info ValueRewritten minecraft:entity/component_groups/g/minecraft:pushable_by_block
  Info DroppedField minecraft:entity/component_groups/g/minecraft:pushable_by_block/unexpected_field
  Error DocumentRejected 
1.26.20 refused
  Error UnrecognizedComponent minecraft:entity/component_groups/g/demo:made_up
  Error UnrecognizedField minecraft:entity/component_groups/g/minecraft:pushable_by_block/unexpected_field
  Info ValueRewritten minecraft:entity/component_groups/g/minecraft:pushable_by_block
  Info DroppedField minecraft:entity/component_groups/g/minecraft:pushable_by_block/unexpected_field
  Error DocumentRejected 
1.26.50 refused
  Error UnrecognizedComponent minecraft:entity/component_groups/g/demo:made_up
  Error UnrecognizedField minecraft:entity/component_groups/g/minecraft:pushable_by_block/unexpected_field
  Info ValueRewritten minecraft:entity/component_groups/g/minecraft:pushable_by_block
  Info DroppedField minecraft:entity/component_groups/g/minecraft:pushable_by_block/unexpected_field
  Error DocumentRejected 
1.26.60 refused
  Error UnrecognizedComponent minecraft:entity/component_groups/g/demo:made_up
  Error UnrecognizedField minecraft:entity/component_groups/g/minecraft:pushable_by_block/unexpected_field
  Error DocumentRejected 
beta refused
  Error UnrecognizedComponent minecraft:entity/component_groups/g/demo:made_up
  Error UnrecognizedField minecraft:entity/component_groups/g/minecraft:pushable_by_block/unexpected_field
  Error DocumentRejected 
",
        "\n{found}"
    );
}

/// A deprecated component is told in the validate phase (cereal's "not present in the
/// Schema" line), graded by the document's version.
#[test]
fn a_deprecated_component_is_graded_by_the_version() {
    let found = told(
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:foot_size": {"value": 1}}}"#,
    );
    assert_eq!(
        found,
        r"1.21.88 loaded
  Warning UnrecognizedComponent minecraft:entity/components/minecraft:foot_size
1.21.89 refused
  Error UnrecognizedComponent minecraft:entity/components/minecraft:foot_size
  Error DocumentRejected 
1.26.20 refused
  Error UnrecognizedComponent minecraft:entity/components/minecraft:foot_size
  Error DocumentRejected 
1.26.50 refused
  Error UnrecognizedComponent minecraft:entity/components/minecraft:foot_size
  Error DocumentRejected 
1.26.60 refused
  Error UnrecognizedComponent minecraft:entity/components/minecraft:foot_size
  Error DocumentRejected 
beta refused
  Error UnsupportedVersion 
",
        "\n{found}"
    );
}

/// R1 of the grading note: a member inside `description/entity_version` that its hand checks
/// do not read, and the `entity_version` and `upgrades` members themselves below 1.26.20.
/// The entity reader's lift leaves the findings strictly below `entity_version` alone and the
/// pipeline's does not; the pin shows there is no warning there for either to grade: below
/// 1.26.20 only the member itself is told, and from 1.26.20 on the members below it are
/// information.
#[test]
fn an_unread_member_of_entity_version_is_graded_by_the_version() {
    let found = told(
        r#"{"description": {"identifier": "demo:x", "entity_version": {"upgrades": [{"upgrade_to": 1, "name": "h", "zz": 3}], "yy": 4}}, "components": {}, "upgrades": {"h": {}}}"#,
    );
    assert_eq!(
        found,
        r"1.21.88 loaded
  Warning UnrecognizedField minecraft:entity/description/entity_version
  Warning UnrecognizedField minecraft:entity/upgrades
1.21.89 refused
  Error UnrecognizedField minecraft:entity/description/entity_version
  Error UnrecognizedField minecraft:entity/upgrades
  Error DocumentRejected 
1.26.20 loaded
  Info MissingToggles minecraft:entity/description/entity_version
  Info UnrecognizedField minecraft:entity/description/entity_version/yy
  Info UnrecognizedField minecraft:entity/description/entity_version/upgrades/0/zz
1.26.50 loaded
  Info UnrecognizedField minecraft:entity/description/entity_version/yy
  Info UnrecognizedField minecraft:entity/description/entity_version/upgrades/0/zz
1.26.60 loaded
  Info UnrecognizedField minecraft:entity/description/entity_version/yy
  Info UnrecognizedField minecraft:entity/description/entity_version/upgrades/0/zz
beta refused
  Info UnrecognizedField minecraft:entity/description/entity_version/yy
  Info UnrecognizedField minecraft:entity/description/entity_version/upgrades/0/zz
  Error UnsupportedVersion 
",
        "\n{found}"
    );
}
