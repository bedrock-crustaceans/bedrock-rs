//! Fixtures `a01`–`a19` and `a30`: the version gate and chain of an
//! attachable and the shapes of `description.item`. The Molang
//! rows `am01`–`am06` are in `molang.rs`, the stack rows `a20`–`a29` in `bindings.rs`, the
//! rows that need the item index (`a17`, `a18`) in `link.rs`. Resource pack at
//! `min_engine_version` 1.21.0 unless the row says otherwise.
//!
//! The files under `fixtures/item/attachable/vanilla/` are copied from the game's sample
//! resource pack (1.26.50), `resource_pack/attachables/`.

use bedrock_addon::diagnostic::{Diagnostics, Disposition, JsonPathSegment, Kind, Severity};
use bedrock_addon::document::{Content, Placed};
use bedrock_addon::families::attachable::{Attachable, Condition, ItemBinding};
use bedrock_addon::id::AttachableId;
use bedrock_addon::molang::{Molang, pack_molang_version};
use bedrock_addon::version::{ENGINE_VERSION, FormatVersion, MinEngineVersion};
use bedrock_addon::visual::HeldItemScale;

use super::common::*;

/// Requires no finding of `kind`.
#[track_caller]
fn none_of_kind(d: &Diagnostics, kind: Kind) {
    assert!(
        d.iter().all(|f| f.kind() != kind),
        "{}",
        lines(d).join("\n")
    );
}

const FAMILY: &str = "attachable";
const ITEM_AT: &str = "minecraft:attachable/description/item";
const NOT_VALID: &str = "core/child_not_valid";
const TYPE_ERROR: &str = "unknown child schema option type.  Allowed types: ";
const VERSION_NOT_GIVEN: &str = "Warning: The \"format_version\" field was not given. Fields from format version 1.8.0 and older will be loaded. Update your targeted \"format_version\" to use fields from newer versions.";

/// The two versions whose structs declare `item`.
const WITH_ITEM: [&str; 2] = ["1.10.0", "1.26.0"];

fn child(name: &str) -> String {
    format!("child '{name}' not valid here.")
}

/// The text of a binding's Molang condition.
fn condition_text(condition: &Condition) -> &str {
    match condition {
        Condition::Molang(source) => source.as_str(),
        other => panic!("{other:?}"),
    }
}

// ---- version gate and chain ----

#[test]
fn a01_vanilla_1_8_0_armor() {
    let text = fixture(FAMILY, "vanilla/diamond_helmet.json");
    let document = loaded(attachable(&text, &cx()));
    clean(document.diagnostics());
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 8, 0));
    assert!(document.version().upgraded);
    assert_eq!(document.format_version, FormatVersion::new(1, 26, 0));
    assert_eq!(document.identifier().as_str(), "minecraft:diamond_helmet");
    // No `item`: the attachable is drawn for the item of its own identifier.
    assert_eq!(document.description.item, None);
    assert_eq!(document.binding(), None);
    assert!(document.bound_items().is_empty());
    let scripts = document.description.slots.scripts.as_ref().unwrap();
    assert!(scripts.parent_setup.is_some());
}

#[test]
fn a02_vanilla_1_10_0_player_override() {
    let text = fixture(FAMILY, "vanilla/diamond_helmet.player.json");
    let document = loaded(attachable(&text, &cx()));
    clean(document.diagnostics());
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 10, 0));
    assert_eq!(
        document.identifier().as_str(),
        "minecraft:diamond_helmet.player"
    );
    let (item, condition) = document.binding().unwrap();
    assert_eq!(item.as_str(), "minecraft:diamond_helmet");
    assert_eq!(
        condition_text(&condition),
        "query.owner_identifier == 'minecraft:player'"
    );
    assert!(!condition.is_always() && !condition.is_never());
    // The identifier need not be the file's name, and the default path follows the
    // identifier: a free key has no namespace folder.
    assert_eq!(
        document.default_path().as_str(),
        "attachables/minecraft_diamond_helmet.player.json"
    );
}

#[test]
fn a03_vanilla_partial_version() {
    let text = fixture(FAMILY, "vanilla/trident.entity.json");
    assert!(text.contains(r#""format_version": "1.10","#));
    let document = loaded(attachable(&text, &cx()));
    clean(document.diagnostics());
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 10, 0));
    assert_eq!(document.identifier().as_str(), "minecraft:trident");
}

#[test]
fn a04_1_26_0_latest() {
    let text = fixture(FAMILY, "a04_1_26_0_latest.json");
    let document = loaded(attachable(&text, &cx()));
    clean(document.diagnostics());
    // Parsed by the newest struct: no step runs.
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 26, 0));
    assert!(!document.version().upgraded);
    assert_eq!(document.held_item_scale(), 0.5);
    assert_eq!(
        document.description.slots.held_item_scale,
        Some(HeldItemScale::Number(0.5))
    );
    assert_eq!(
        document.initialize().as_deref(),
        Some("variable.charge = 0;variable.glow = 1;")
    );
    let (item, condition) = document.binding().unwrap();
    assert_eq!(item.as_str(), "example:wand");
    assert!(condition.is_always());
    // Written back, it is the document that was read.
    let written = round_trip(&document, |text| attachable(text, &cx()));
    let json = |text: &str| -> bedrock_addon::json::Value { facet_json::from_str(text).unwrap() };
    assert_eq!(json(&written), json(&text));
}

/// `text` without its `format_version` member.
fn without_version(text: &str) -> String {
    let line = text
        .lines()
        .find(|line| line.contains("\"format_version\""))
        .unwrap();
    text.replacen(line, "", 1)
}

#[test]
fn a05_missing_version_deviation() {
    // Vanilla: `absent format_version: 1.0.0, legacy root-key
    // path, becomes client entity minecraft:minecraft:attachable with no fields"`: read as
    // 1.8.0, as the attachable it names.
    let text = without_version(&fixture(FAMILY, "vanilla/diamond_helmet.json"));
    let document = loaded(attachable(&text, &cx()));
    expect(
        document.diagnostics(),
        "core/version_not_given_legacy_fields",
        Severity::Warning,
        VERSION_NOT_GIVEN,
    );
    assert_eq!(document.version().written, None);
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 8, 0));
    assert_eq!(document.identifier().as_str(), "minecraft:diamond_helmet");
    assert_eq!(document.binding(), None);
    // The warning is the only finding at warning level.
    assert_eq!(
        document.diagnostics().at_least(Severity::Warning).count(),
        1
    );
}

#[test]
fn a05_missing_version_with_item_is_rejected_as_any_1_8_document() {
    let text = without_version(&fixture(FAMILY, "vanilla/diamond_helmet.player.json"));
    let d = rejected(attachable(&text, &cx()));
    expect(
        &d,
        "core/version_not_given_legacy_fields",
        Severity::Warning,
        VERSION_NOT_GIVEN,
    );
    expect(&d, NOT_VALID, Severity::Error, &child("item"));
    // The warning comes first.
    let position = |name: &str| d.iter().position(|f| f.code.id() == name).unwrap();
    assert!(position("core/version_not_given_legacy_fields") < position(NOT_VALID));
}

#[test]
fn a06_below_floor() {
    let text = fixture(FAMILY, "vanilla/diamond_helmet.json").replace("1.8.0", "1.7.0");
    let d = rejected(attachable(&text, &cx()));
    expect(
        &d,
        "core/version_below_min",
        Severity::Error,
        "Version 1.7.0 below min required version (1.10.0)",
    );
    assert_eq!(d.len(), 1);
}

#[test]
fn a07_major_zero() {
    let text = ghost("0.9.0", "");
    let outcome = Attachable::parse(
        &bedrock_addon::storage::PackPath::from_static("attachables/a07.json"),
        text.as_bytes(),
        &cx(),
    );
    let d = rejected(outcome);
    expect(
        &d,
        "visual/unknown_version",
        Severity::Error,
        "Unknown/unsupported version value specified in format_version field in attachables/a07.json",
    );
}

#[test]
fn a08_above_engine() {
    let d = rejected(attachable(&ghost("99.0.0", ""), &cx()));
    assert_eq!(ENGINE_VERSION, FormatVersion::new(1, 26, 60));
    expect(
        &d,
        "visual/actor_above_engine",
        Severity::Error,
        "Error: actor resource file has format version: 99.0.0 exceeds game version: 1.26.60. Skipping.",
    );
}

#[test]
fn a09_beta_literal() {
    // `Beta = Infallible`: the literal is no version, with or without the Beta APIs.
    let mut beta = cx();
    beta.beta_apis = true;
    beta.is_base_game_pack = true;
    for cx in [cx(), beta] {
        let d = rejected(attachable(&ghost("beta", ""), &cx));
        expect(
            &d,
            "visual/unknown_version",
            Severity::Error,
            "Unknown/unsupported version value specified in format_version field in attachables/x.json",
        );
        none_of_kind(&d, Kind::Toggle);
    }
}

const PRE_ANIMATION: &str =
    r#""scripts": {"pre_animation": ["Variable.Charge = Query.Is_Using_Item;"]}"#;

#[test]
fn a11_script_lowercase_gate() {
    // Just below 1.18.20 the stored text is lower-cased, as the loader does before it parses.
    let below = loaded(attachable(&ghost("1.18.10", PRE_ANIMATION), &cx()));
    assert_eq!(
        below.pre_animation().as_deref(),
        Some("variable.charge = query.is_using_item;")
    );
    clean(below.diagnostics());
    // At 1.18.20 it is kept as written.
    let at = loaded(attachable(&ghost("1.18.20", PRE_ANIMATION), &cx()));
    assert_eq!(
        at.pre_animation().as_deref(),
        Some("Variable.Charge = Query.Is_Using_Item;")
    );
    // (`InvalidMolang` too when the `molang` feature is on.) Molang names
    // compare without case, so the text compiles as written.
    clean(at.diagnostics());
}

#[test]
fn a11_an_item_condition_is_not_lower_cased() {
    // Script lowercasing: the `item` value goes straight to the expression.
    let members = r#""item": {"Example:Wand": "Query.Is_Sneaking"}"#;
    let document = loaded(attachable(&ghost("1.10.0", members), &cx()));
    let (item, condition) = document.binding().unwrap();
    assert_eq!(item.as_str(), "Example:Wand");
    assert_eq!(condition_text(&condition), "Query.Is_Sneaking");
}

// ---- `item` shapes ----

#[test]
fn a12_item_string_form() {
    let members = r#""item": "example:wand""#;
    // Rule 14a: no member of 1.8.0, in either form.
    for version in ["1.8.0", "1.9.9"] {
        let d = rejected(attachable(&ghost(version, members), &cx()));
        expect(&d, NOT_VALID, Severity::Error, &child("item"));
        expect_at(&d, NOT_VALID, Severity::Error, ITEM_AT);
        assert_eq!(d.len(), 1);
    }
    for version in WITH_ITEM {
        let document = loaded(attachable(&ghost(version, members), &cx()));
        silent(document.diagnostics());
        let (item, condition) = document.binding().unwrap();
        assert_eq!(item.as_str(), "example:wand");
        assert_eq!(condition, Condition::Always);
        assert!(condition.is_always() && !condition.is_never());
        assert_eq!(
            document.description.item,
            Some(ItemBinding::Item("example:wand".to_owned()))
        );
        let item = document.description.item.as_ref().unwrap();
        assert_eq!(item.ignored().count(), 0);
    }
}

#[test]
fn a12_the_string_form_is_bound_as_written() {
    // Not lower-cased, no `minecraft:` added: vanilla compares bytes.
    for written in ["Minecraft:Bow", "bow", "a:b:c"] {
        let members = format!(r#""item": "{written}""#);
        let document = loaded(attachable(&ghost("1.10.0", &members), &cx()));
        assert_eq!(document.binding().unwrap().0.as_str(), written);
    }
}

#[test]
fn a13_item_multi_member_binds_the_smallest_key_deviation() {
    // Vanilla: std::map<CZString> order (strcmp): the member read is the smallest
    // key by bytes, not the first written; ours adds the warnings.
    let members = r#""item": {"example:b": "1.0", "example:a": "q.is_sneaking"}"#;
    for version in WITH_ITEM {
        let document = loaded(attachable(&ghost(version, members), &cx()));
        let (item, condition) = document.binding().unwrap();
        assert_eq!(item.as_str(), "example:a");
        assert_eq!(condition_text(&condition), "q.is_sneaking");
        expect_at(
            document.diagnostics(),
            "core/dropped_field",
            Severity::Info,
            "minecraft:attachable/description/item/example:b",
        );
        assert_eq!(
            document.diagnostics().at_least(Severity::Warning).count(),
            0
        );
        // Both members are kept as written.
        let item = document.description.item.as_ref().unwrap();
        assert!(matches!(item, ItemBinding::Conditional(members) if members.len() == 2));
        assert_eq!(item.ignored().collect::<Vec<_>>(), ["example:b"]);
    }
}

#[test]
fn a13_bytes_decide_which_member_binds_and_every_other_is_dropped() {
    // `B` (0x42) sorts before `a` (0x61); three members leave two dropped.
    let members = r#""item": {"example:a": "1.0", "example:c": "1.0", "example:B": "2.0"}"#;
    let document = loaded(attachable(&ghost("1.10.0", members), &cx()));
    assert_eq!(document.binding().unwrap().0.as_str(), "example:B");
    let dropped: Vec<String> = document
        .diagnostics()
        .iter()
        .filter(|f| f.code.id() == "core/dropped_field")
        .map(|f| {
            assert_eq!(f.severity, Severity::Info);
            f.location.json_path.to_string()
        })
        .collect();
    assert_eq!(
        dropped,
        [
            "minecraft:attachable/description/item/example:a",
            "minecraft:attachable/description/item/example:c"
        ]
    );
    // One member: nothing dropped.
    let one = loaded(attachable(
        &ghost("1.10.0", r#""item": {"example:a": "1.0"}"#),
        &cx(),
    ));
    none_of_kind(one.diagnostics(), Kind::UnknownMember(Disposition::Dropped));
}

#[test]
fn a14_item_empty_object() {
    for version in WITH_ITEM {
        let d = rejected(attachable(&ghost(version, r#""item": {}"#), &cx()));
        expect(
            &d,
            "core/required_child",
            Severity::Error,
            "Required child [a-zA-Z0-9_.:-]+ not found",
        );
        expect_at(&d, "core/required_child", Severity::Error, ITEM_AT);
        assert_eq!(d.len(), 1, "{}", lines(&d).join("\n"));
    }
}

#[test]
fn a15_item_wrong_type() {
    for version in WITH_ITEM {
        for value in [
            "5",
            "null",
            "[]",
            "true",
            "1.5",
            "-1",
            r#"["example:wand"]"#,
        ] {
            let members = format!(r#""item": {value}"#);
            let d = rejected(attachable(&ghost(version, &members), &cx()));
            expect(
                &d,
                "attachable/unknown_item_type",
                Severity::Error,
                &format!("{TYPE_ERROR}string, object"),
            );
            expect_at(&d, "attachable/unknown_item_type", Severity::Error, ITEM_AT);
            assert_eq!(d.len(), 1, "{value}: {}", lines(&d).join("\n"));
        }
    }
}

#[test]
fn a16_item_non_string_condition() {
    for version in WITH_ITEM {
        // A number passes the schema and reads as the empty text: never true.
        for number in ["1", "0", "1.5", "-2"] {
            let members = format!(r#""item": {{"example:wand": {number}}}"#);
            let document = loaded(attachable(&ghost(version, &members), &cx()));
            expect_at(
                document.diagnostics(),
                "core/invalid_value",
                Severity::Info,
                "minecraft:attachable/description/item/example:wand",
            );
            assert_eq!(
                document.diagnostics().at_least(Severity::Warning).count(),
                0
            );
            let (item, condition) = document.binding().unwrap();
            assert_eq!(item.as_str(), "example:wand");
            assert_eq!(condition_text(&condition), "");
            assert!(condition.is_never() && !condition.is_always(), "{number}");
        }
        // A boolean reads as its text and is said nothing about.
        for (boolean, text) in [("true", "true"), ("false", "false")] {
            let members = format!(r#""item": {{"example:wand": {boolean}}}"#);
            let document = loaded(attachable(&ghost(version, &members), &cx()));
            clean(document.diagnostics());
            none_of_kind(document.diagnostics(), Kind::InvalidValue);
            let (_, condition) = document.binding().unwrap();
            assert_eq!(condition_text(&condition), text);
            assert!(!condition.is_never());
        }
    }
}

#[test]
fn a16_only_the_member_that_binds_is_warned_about_as_a_number() {
    // `example:b` is dropped (rule 17); its number binds nothing and gets no second finding.
    let members = r#""item": {"example:b": 1, "example:a": "q.is_sneaking"}"#;
    let document = loaded(attachable(&ghost("1.10.0", members), &cx()));
    none_of_kind(document.diagnostics(), Kind::InvalidValue);
    // the drop is an information: nothing at warning level
    assert_eq!(
        document.diagnostics().at_least(Severity::Warning).count(),
        0
    );
    // The other way round the number binds.
    let members = r#""item": {"example:b": "q.is_sneaking", "example:a": 1}"#;
    let document = loaded(attachable(&ghost("1.10.0", members), &cx()));
    expect_at(
        document.diagnostics(),
        "core/invalid_value",
        Severity::Info,
        "minecraft:attachable/description/item/example:a",
    );
    assert!(document.binding().unwrap().1.is_never());
}

#[test]
fn a16_a_condition_that_is_no_number_boolean_or_string_fails_the_file() {
    // `item` (object form): "null, array, object → unknown child schema option
    // type"; every member is validated, the one that binds or not.
    for version in WITH_ITEM {
        for value in ["null", "[]", "{}", r#"["1.0"]"#] {
            for (members, key) in [
                (
                    format!(r#""item": {{"example:wand": {value}}}"#),
                    "example:wand",
                ),
                (
                    format!(r#""item": {{"example:a": "1.0", "example:b": {value}}}"#),
                    "example:b",
                ),
            ] {
                let d = rejected(attachable(&ghost(version, &members), &cx()));
                expect(
                    &d,
                    "core/unknown_child_option",
                    Severity::Error,
                    &format!("{TYPE_ERROR}molang"),
                );
                expect_at(
                    &d,
                    "core/unknown_child_option",
                    Severity::Error,
                    &format!("{ITEM_AT}/{key}"),
                );
            }
        }
    }
}

#[test]
fn a17_item_case_mismatch_the_binding_is_kept_as_written() {
    // The `UnresolvedReference` of rule 22 needs the item index: `link.rs`. Loading keeps the
    // binding (which never matches) and says nothing.
    let members = r#""item": {"Minecraft:Bow": "1"}"#;
    let document = loaded(attachable(&ghost("1.10.0", members), &cx()));
    clean(document.diagnostics());
    assert_eq!(document.binding().unwrap().0.as_str(), "Minecraft:Bow");
}

#[test]
fn a17b_item_bad_key() {
    for version in WITH_ITEM {
        let members = r#""item": {"example:wand sword": "1"}"#;
        let d = rejected(attachable(&ghost(version, members), &cx()));
        expect(&d, NOT_VALID, Severity::Error, &child("example:wand sword"));
        expect_at(
            &d,
            NOT_VALID,
            Severity::Error,
            "minecraft:attachable/description/item/example:wand sword",
        );
        assert_eq!(d.len(), 1, "{}", lines(&d).join("\n"));
    }
}

#[test]
fn a17b_the_key_pattern_at_its_edges() {
    // `[a-zA-Z0-9_.:-]+` in full: every class, and at least one character.
    let ok = r#""item": {"aZ09_.:-": "1"}"#;
    clean(loaded(attachable(&ghost("1.10.0", ok), &cx())).diagnostics());
    // Each key outside it is the file's one finding: an error at the key, with vanilla's
    // text naming it.
    for bad in ["", "a b", "a/b", "a+b", "é", "wand!", "@a", "a,b"] {
        for version in WITH_ITEM {
            let members = format!(r#""item": {{"{bad}": "1"}}"#);
            let d = rejected(attachable(&ghost(version, &members), &cx()));
            // The path writes a `/` inside the key as `~1`; the game's text names the key.
            let key = bad.replace('/', "~1");
            assert_eq!(
                lines(&d),
                [format!(
                    "Error {NOT_VALID} {ITEM_AT}/{key} child '{bad}' not valid here."
                )],
                "{version} {bad:?}"
            );
            // The key is one segment of the path, whatever it holds (`a/b` too): the
            // fourth, below `item`.
            let path = &d.iter().next().unwrap().location.json_path;
            assert_eq!(path.len(), 4, "{version} {bad:?}");
            assert_eq!(
                path.segments().last(),
                Some(JsonPathSegment::Key(bad)),
                "{version} {bad:?}"
            );
        }
    }
    // A member that does not bind (it sorts after the one that does) is checked as well:
    // the error, and the note that the member is not the one read.
    let members = r#""item": {"example:a": "1", "example:b c": "1"}"#;
    let d = rejected(attachable(&ghost("1.10.0", members), &cx()));
    assert_eq!(
        lines(&d),
        [
            format!("Error {NOT_VALID} {ITEM_AT}/example:b c child 'example:b c' not valid here."),
            format!("Info core/dropped_field {ITEM_AT}/example:b c field dropped"),
        ]
    );
    // And one that would bind, beside a member that is fine.
    let members = r#""item": {"example:a": "1", "example b": "1"}"#;
    let d = rejected(attachable(&ghost("1.10.0", members), &cx()));
    assert_eq!(
        lines(&d),
        [
            format!("Error {NOT_VALID} {ITEM_AT}/example b child 'example b' not valid here."),
            format!("Info core/dropped_field {ITEM_AT}/example:a field dropped"),
        ]
    );
}

#[test]
fn a17b_a_string_form_outside_the_pattern_is_warned_about_and_kept() {
    // Rule 20, string form: vanilla takes any string.
    for version in WITH_ITEM {
        for written in ["example:wand sword", "", "wand!"] {
            let members = format!(r#""item": "{written}""#);
            let document = loaded(attachable(&ghost(version, &members), &cx()));
            expect_at(
                document.diagnostics(),
                "core/invalid_value",
                Severity::Info,
                ITEM_AT,
            );
            assert_eq!(
                document.diagnostics().at_least(Severity::Warning).count(),
                0
            );
            let (item, condition) = document.binding().unwrap();
            assert_eq!(item.as_str(), written);
            assert!(condition.is_always());
        }
        // Every character of the pattern is fine.
        let document = loaded(attachable(&ghost(version, r#""item": "aZ09_.:-""#), &cx()));
        silent(document.diagnostics());
    }
}

#[test]
fn a19_item_molang_version() {
    let text = fixture(FAMILY, "vanilla/diamond_helmet.player.json");
    let mut seen = Vec::new();
    for (a, b, c) in [(1, 16, 0), (1, 21, 100)] {
        let document = loaded(attachable(&text, &rp(a, b, c)));
        let expected = pack_molang_version(Some(&MinEngineVersion::new(a, b, c)));
        let Some(ItemBinding::Conditional(members)) = &document.description.item else {
            panic!("{:?}", document.description.item);
        };
        let Molang::Expr(source) = &members["minecraft:diamond_helmet"] else {
            panic!("{members:?}");
        };
        assert_eq!(source.raw_version(), Some(expected));
        // The binding holds the same source.
        let (_, condition) = document.binding().unwrap();
        let Condition::Molang(bound) = condition else {
            panic!("{condition:?}");
        };
        assert_eq!(bound.raw_version(), Some(expected));
        seen.push(expected);
    }
    // The two packs are on either side of a Molang version: never the document's 1.10.0.
    assert_ne!(seen[0], seen[1]);
}

#[test]
fn a19_every_item_member_takes_the_pack_s_version() {
    let members = r#""item": {"example:b": "q.is_sneaking", "example:a": "q.is_baby"}"#;
    for version in WITH_ITEM {
        let document = loaded(attachable(&ghost(version, members), &rp(1, 16, 0)));
        let expected = pack_molang_version(Some(&MinEngineVersion::new(1, 16, 0)));
        let Some(ItemBinding::Conditional(members)) = &document.description.item else {
            panic!("{:?}", document.description.item);
        };
        for (key, value) in members {
            let Molang::Expr(source) = value else {
                panic!("{key}");
            };
            assert_eq!(source.raw_version(), Some(expected), "{key}");
        }
    }
}

#[test]
fn a30_unknown_member_rejected() {
    let members = r#""held_item_scale": 0.5"#;
    for version in ["1.8.0", "1.10.0", "1.25.99"] {
        let d = rejected(attachable(&ghost(version, members), &cx()));
        expect(&d, NOT_VALID, Severity::Error, &child("held_item_scale"));
        assert_eq!(d.len(), 1);
    }
    let document = loaded(attachable(&ghost("1.26.0", members), &cx()));
    silent(document.diagnostics());
    assert_eq!(document.held_item_scale(), 0.5);
    // `offset` is an animation key in vanilla's files, never a description member.
    for version in ["1.8.0", "1.10.0", "1.26.0"] {
        let d = rejected(attachable(&ghost(version, r#""offset": [0, 1, 0]"#), &cx()));
        expect(&d, NOT_VALID, Severity::Error, &child("offset"));
    }
    // `animation_controllers` is no member from 1.10.0.
    for version in ["1.10.0", "1.26.0"] {
        let d = rejected(attachable(
            &ghost(version, r#""animation_controllers": []"#),
            &cx(),
        ));
        expect(
            &d,
            NOT_VALID,
            Severity::Error,
            &child("animation_controllers"),
        );
    }
}

#[test]
fn the_default_path_is_the_identifier_with_its_colons_as_underscores() {
    for (identifier, path) in [
        ("demo:ghost", "attachables/demo_ghost.json"),
        ("ghost", "attachables/ghost.json"),
        ("a:b:c", "attachables/a_b_c.json"),
    ] {
        let attachable = Attachable::new(AttachableId::parse(identifier));
        assert_eq!(attachable.default_path().as_str(), path);
    }
}
