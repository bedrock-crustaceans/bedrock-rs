//! Fixture `a10`: what the 1.8 step does to an attachable, which is
//! not what it does to a client entity.

#![allow(
    clippy::disallowed_methods,
    reason = "calls facet-json directly: parses a fixture or written text into a json Value to compare it; not yet moved to the door"
)]

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::JsonDocument as _;
use bedrock_addon::families::attachable::Attachable;
use bedrock_addon::families::attachable::history::{self, AttachableHistory};
use bedrock_addon::json::Value;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::FormatVersion;
use bedrock_addon::versioning::VersionStruct;

use super::common::*;

const FAMILY: &str = "attachable";
const CONTROLLERS_AT: &str = "minecraft:attachable/description/animation_controllers";

fn json(text: &str) -> Value {
    facet_json::from_str(text).expect("test JSON")
}

#[test]
fn a10_1_8_controllers_rejected_deviation() {
    // Vanilla does not convert it (its converter only visits minecraft:client_entity) and
    // then rejects it by the strict v1_10 schema: the same rejection, raised by the step.
    let before = fixture(FAMILY, "a10_wand.before.json");
    let with = before.replace(
        r#""render_controllers""#,
        r#""animation_controllers": [ { "hold": "controller.animation.wand.hold" } ],
      "render_controllers""#,
    );
    assert_ne!(with, before);
    let d = rejected(attachable(&with, &cx()));
    expect(
        &d,
        "core/child_not_valid",
        Severity::Error,
        "child 'animation_controllers' not valid here.",
    );
    expect_at(&d, "core/child_not_valid", Severity::Error, CONTROLLERS_AT);
    assert_eq!(d.len(), 1, "{}", lines(&d).join("\n"));
    // With or without `animations` beside it, and for an empty list too: the member is
    // what 1.10.0 lacks.
    for members in [
        r#""animation_controllers": [{"hold": "controller.animation.wand.hold"}]"#,
        r#""animation_controllers": []"#,
        r#""animations": {}, "animation_controllers": []"#,
    ] {
        for version in ["1.8.0", "1.9.9"] {
            let d = rejected(attachable(&ghost(version, members), &cx()));
            expect_at(&d, "core/child_not_valid", Severity::Error, CONTROLLERS_AT);
        }
    }
    // A document without a version is a 1.8.0 one.
    let text = r#"{"minecraft:attachable": {"description": {"identifier": "demo:ghost", "animation_controllers": []}}}"#;
    let d = rejected(attachable(text, &cx()));
    expect_at(&d, "core/child_not_valid", Severity::Error, CONTROLLERS_AT);
}

#[test]
fn a10_the_list_is_checked_by_the_1_8_schema_before_the_step_rejects_it() {
    // The v1_8 schema declares the member: a wrong shape in it is that schema's finding.
    let d = rejected(attachable(
        &ghost("1.8.0", r#""animation_controllers": [{"my:hold": "x"}]"#),
        &cx(),
    ));
    expect(
        &d,
        "core/child_not_valid",
        Severity::Error,
        "child 'my:hold' not valid here.",
    );
    let d = rejected(attachable(
        &ghost("1.8.0", r#""animation_controllers": {}"#),
        &cx(),
    ));
    assert!(find(&d, "core/unknown_child_option", Severity::Error).is_some());
}

#[test]
fn a10_without_the_list_the_step_only_stamps_the_version() {
    let before = fixture(FAMILY, "a10_wand.before.json");
    let after = fixture(FAMILY, "a10_wand.after.json");
    let path = PackPath::from_static(PATH);
    let written = loaded(Attachable::parse_as_written(
        &path,
        before.as_bytes(),
        &cx(),
    ));
    let AttachableHistory::V1_8_0(old) = written.value().clone() else {
        panic!("{:?}", written.value());
    };
    // As written, it is the before-document.
    assert_eq!(old.write().unwrap(), json(&before));
    let mut diagnostics = bedrock_addon::diagnostic::Diagnostics::default();
    let mut upgrade = bedrock_addon::versioning::UpgradeCx {
        load: &cx(),
        original_version: FormatVersion::new(1, 8, 0),
        identifier: "example:wand",
        actor_type: bedrock_addon::vanilla::ActorType::Mob,
        skip_section_a: false,
        reading: bedrock_addon::versioning::Reading::OWN,
        diagnostics: &mut diagnostics,
    };
    let stepped = bedrock_addon::versioning::Upgrade::upgrade(old, &mut upgrade);
    // Vanilla stamps "1.10"; a version is written in full (as the client entity step does).
    assert_eq!(stepped.write().unwrap(), json(&after));
    silent(&diagnostics);
    // The whole load goes on to 1.26.0 and converts nothing: no `controller__` animation,
    // no `animate`.
    let document = loaded(history::upgrade(written, &cx()));
    clean(document.diagnostics());
    assert_eq!(document.format_version, FormatVersion::new(1, 26, 0));
    let animations = document.description.slots.animations.as_ref().unwrap();
    assert_eq!(animations.len(), 1);
    assert_eq!(animations["hold"].as_str(), "animation.wand.hold");
    let scripts = document.description.slots.scripts.as_ref().unwrap();
    assert_eq!(scripts.animate, None);
    assert!(scripts.parent_setup.is_some());
}

/// An unknown member put in the table of a document as written (an edit) is upgraded with its
/// object, and the upgrade's steps see it as they see a member of a parse.
#[test]
fn upgrade_keeps_the_unknown_members_of_the_document_as_written() {
    use bedrock_addon::diagnostic::JsonPath;
    let before = fixture(FAMILY, "a10_wand.before.json");
    let path = PackPath::from_static(PATH);
    let mut written = loaded(Attachable::parse_as_written(
        &path,
        before.as_bytes(),
        &cx(),
    ));
    let description = JsonPath::of_keys(["minecraft:attachable", "description"]);
    written.unknown_mut().put(
        &description,
        "zz_description",
        bedrock_addon::json::Value::from(1i64),
    );
    let expected = written.unknown().clone();
    let upgraded = loaded(history::upgrade(written, &cx()));
    assert_eq!(upgraded.unknown(), &expected);
}
