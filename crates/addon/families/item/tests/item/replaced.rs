//! What a step or the legacy reader of an item replaces or drops is told at the member the
//! FILE has, or kept: a `use_modifiers`
//! or `minecraft:tags` no slice of the file's version binds, the members of a component a step
//! takes apart, a `tag:` key beside `minecraft:tags`, and the legacy `creative_category`.

use bedrock_addon_core::load::LoadContext;

use super::common::{allowlisted, item, loads, newest_payload, rejects, third_party};

/// The findings of the item with `components` at `version` as `Code Severity at path`
/// (below `minecraft:item/`), and the written text on one line.
#[track_caller]
fn told(version: &str, components: &str, cx: &LoadContext) -> (Vec<String>, String) {
    let mut document = loads(&item(version, "demo:x", components), cx);
    // An item the steps leave without a component the game's routing test knows has no text: such a document is read with a stack size beside its components.
    if refused_at_components(&document) {
        let components = format!(r#"{components}, "minecraft:max_stack_size": 16"#);
        document = loads(&item(version, "demo:x", &components), cx);
    }
    let found = document
        .diagnostics()
        .iter()
        .map(|f| {
            let code: String = f.code.id().to_owned();
            let at = f.location.json_path.to_string();
            let at = at.strip_prefix("minecraft:item/").unwrap_or(&at).to_owned();
            format!("{code} {:?} at {at}", f.severity)
        })
        .collect();
    let written = written_or_refused(&document);
    (
        found,
        written.split_whitespace().collect::<Vec<_>>().join(" "),
    )
}

/// Whether the newest writer refuses `document` at `components`: an item left without a
/// registered component.
fn refused_at_components(
    document: &bedrock_addon_core::document::Document<bedrock_addon_family_item::Item>,
) -> bool {
    matches!(
        document.to_json(),
        Err(bedrock_addon_core::json::EncodeError::NotRepresentable { at, .. })
            if at.to_string() == "minecraft:item/components"
    )
}

/// The newest writer's text of `document`; where it refuses the item (a member the strict
/// 1.26.50 format does not bind), `REFUSED at <place>:` and the payload the
/// newest struct holds (what the steps carried).
fn written_or_refused(
    document: &bedrock_addon_core::document::Document<bedrock_addon_family_item::Item>,
) -> String {
    match document.to_json() {
        Ok(written) => written,
        Err(bedrock_addon_core::json::EncodeError::NotRepresentable { at, .. }) => {
            format!("REFUSED at {at}: {}", newest_payload(document))
        }
        Err(error) => panic!("the item writes: {error:?}"),
    }
}

#[track_caller]
fn has(found: &[String], finding: &str) {
    assert!(
        found.iter().any(|f| f == finding),
        "no `{finding}` in {found:#?}"
    );
}

#[test]
fn case_8_a_use_modifiers_no_slice_of_its_version_binds_is_told_as_not_written() {
    // "No vanilla slice binds `use_modifiers` before 1.20.50 … A
    // `minecraft:use_modifiers` that the author wrote below 1.20.50 reaches only `legacy`."
    // The game's legacy data holds it and so does the item (`legacy.shared`); the written
    // item does not: a warning.
    let (found, written) = told(
        "1.20.40",
        r#""minecraft:use_duration": 3, "minecraft:use_modifiers": {"use_duration": 9, "movement_modifier": 0.5}"#,
        &third_party(),
    );
    has(
        &found,
        "core/dropped_field Info at components/minecraft:use_modifiers",
    );
    assert!(written.contains(r#""use_duration": 3.0"#), "{written}");
    assert!(!written.contains("movement_modifier"), "{written}");
    // Alone too: the item has no such component.
    let (found, written) = told(
        "1.20.40",
        r#""minecraft:use_modifiers": {"use_duration": 9, "movement_modifier": 0.5}"#,
        &third_party(),
    );
    has(
        &found,
        "core/dropped_field Info at components/minecraft:use_modifiers",
    );
    assert!(!written.contains("use_modifiers"), "{written}");
    // From 1.20.50 the component is bound, and the legacy duration is written over its own
    // ("its `use_duration` is overwritten"): noted at the member that goes.
    let (found, written) = told(
        "1.21.0",
        r#""minecraft:use_duration": 2.0, "minecraft:use_modifiers": {"use_duration": 0.5, "movement_modifier": 0.5}"#,
        &third_party(),
    );
    has(
        &found,
        "core/dropped_field Info at components/minecraft:use_modifiers/use_duration",
    );
    assert!(
        written.contains(r#""use_duration": 2.0, "movement_modifier": 0.5"#),
        "{written}"
    );
    // The same duration in both: nothing is replaced, nothing told.
    let (found, _) = told(
        "1.21.0",
        r#""minecraft:use_duration": 0.5, "minecraft:use_modifiers": {"use_duration": 0.5}"#,
        &third_party(),
    );
    assert!(
        !found.iter().any(|f| f.starts_with("core/dropped_field")),
        "{found:#?}"
    );
}

#[test]
fn case_9_a_member_of_use_duration_nothing_reads_is_told_and_kept_where_the_step_rebuilds_it() {
    // 1.20.40: the typed legacy component is rebuilt as `use_modifiers`, its unknown member
    // with it. 1.21.0: the legacy read alone takes the duration; the member is told.
    for (version, value, kept) in [
        ("1.20.40", r#"{"a": 1}"#, true),
        ("1.21.0", r#"{"value": 3, "a": 1}"#, false),
    ] {
        let (found, written) = told(
            version,
            &format!(r#""minecraft:use_duration": {value}"#),
            &third_party(),
        );
        assert_eq!(written.contains(r#""a": 1"#), kept, "{version}: {written}");
        // Told where it is not kept (below the strict versions an unknown member that is
        // kept says nothing, as everywhere in the family).
        assert_eq!(
            found
                .iter()
                .any(|f| f == "core/unrecognized_field Info at components/minecraft:use_duration/a"),
            !kept,
            "{version}: {found:#?}"
        );
    }
}

#[test]
fn case_10_a_tags_component_no_slice_of_its_version_binds_is_noted_beside_a_tag_key() {
    // `tags` is released at 1.20.50; below it the `tag:` keys are the
    // tags.
    for components in [
        r#""tag:a": {}, "minecraft:tags": {"tags": ["b"]}"#,
        r#""minecraft:tags": {"tags": ["b"]}, "tag:a": {}"#,
    ] {
        let (found, written) = told("1.20.40", components, &third_party());
        has(
            &found,
            "core/dropped_field Info at components/minecraft:tags",
        );
        assert!(
            written.contains(r#""minecraft:tags": { "tags": [ "a" ] }"#),
            "{components}: {written}"
        );
    }
}

#[test]
fn case_11_a_legacy_creative_category_the_written_item_has_no_place_for_is_told() {
    let (found, written) = told(
        "1.16.100",
        r#""minecraft:creative_category": {"parent": "itemGroup.name.sword"}, "minecraft:icon": {"texture": "a"}"#,
        &allowlisted(),
    );
    // The game reads the component of a legacy item; the newest format has no member for it
    // and the folder gives no conversion: a value this library loses, a warning.
    has(
        &found,
        "core/dropped_field Info at components/minecraft:creative_category",
    );
    assert!(!written.contains("itemGroup"), "{written}");
}

#[test]
fn case_12_the_unknown_members_of_a_component_a_step_removes_are_told() {
    let (found, written) = told(
        "1.20.30",
        r#""minecraft:on_use": {"zz_a": 1}, "minecraft:weapon": {"zz_b": 1, "on_hurt_entity": {"event": "e"}}, "minecraft:on_use_on": {"zz_c": 2}, "minecraft:icon": {"texture": "a"}"#,
        &allowlisted(),
    );
    has(
        &found,
        "core/unrecognized_field Warning at components/minecraft:on_use/zz_a",
    );
    has(
        &found,
        "core/unrecognized_field Warning at components/minecraft:weapon/zz_b",
    );
    has(
        &found,
        "core/unrecognized_field Warning at components/minecraft:on_use_on/zz_c",
    );
    has(
        &found,
        "core/deprecated_member Info at components/minecraft:weapon/on_hurt_entity",
    );
    for gone in ["zz_a", "zz_b", "zz_c"] {
        assert!(!written.contains(gone), "{gone}: {written}");
    }
    // A chargeable is taken apart by the 1.20.40 step.
    let (found, _) = told(
        "1.20.40",
        r#""minecraft:chargeable": {"movement_modifier": 0.25, "d": 4}, "minecraft:icon": {"texture": "a"}"#,
        &allowlisted(),
    );
    has(
        &found,
        "core/unrecognized_field Warning at components/minecraft:chargeable/d",
    );
}

#[test]
fn a_finding_of_the_modifiers_a_step_made_names_the_member_of_the_file() {
    // Minor 5: the typed read of what the 1.20.40 step made of `use_duration`.
    let (found, _) = told(
        "1.20.40",
        r#""minecraft:use_duration": 4294967296, "minecraft:icon": {"texture": "stick"}"#,
        &third_party(),
    );
    assert!(
        !found.iter().any(|f| f.contains("use_modifiers")),
        "{found:#?}"
    );
    has(
        &found,
        "core/invalid_value Info at components/minecraft:use_duration",
    );
}

#[test]
fn a_legacy_use_duration_that_is_no_number_is_told_from_1_20_50() {
    for (value, at) in [
        (r#""x""#, "components/minecraft:use_duration"),
        ("true", "components/minecraft:use_duration"),
        (
            r#"{"value": [7]}"#,
            "components/minecraft:use_duration/value",
        ),
    ] {
        let (found, written) = told(
            "1.21.0",
            &format!(r#""minecraft:use_duration": {value}"#),
            &third_party(),
        );
        has(&found, &format!("core/invalid_type Info at {at}"));
        assert!(!written.contains("use_modifiers"), "{value}: {written}");
    }
    // `null` is absent in the legacy chain (rule N1): nothing is told, nothing made.
    let (found, written) = told(
        "1.21.0",
        r#""minecraft:use_duration": null"#,
        &third_party(),
    );
    assert!(
        !found.iter().any(|f| f.contains("use_duration")),
        "{found:#?}"
    );
    assert!(!written.contains("use_modifiers"), "{written}");
}

#[test]
fn an_icon_textures_beside_the_legacy_texture_is_noted() {
    let (found, written) = told(
        "1.20.50",
        r#""minecraft:icon": {"texture": "demo_sword", "textures": {"default": "other"}}"#,
        &third_party(),
    );
    has(
        &found,
        "core/dropped_field Info at components/minecraft:icon/textures",
    );
    assert!(written.contains(r#""default": "demo_sword""#), "{written}");
    assert!(!written.contains("other"), "{written}");
}

#[test]
fn a_wearable_protection_the_armor_s_is_written_over_is_noted() {
    let (found, written) = told(
        "1.20.0",
        r#""minecraft:armor": {"protection": 5}, "minecraft:wearable": {"slot": "slot.armor.head", "protection": 2}"#,
        &allowlisted(),
    );
    has(
        &found,
        "core/dropped_field Info at components/minecraft:wearable/protection",
    );
    assert!(written.contains(r#""protection": 5"#), "{written}");
    // The same points: nothing is replaced.
    let (found, _) = told(
        "1.20.0",
        r#""minecraft:armor": {"protection": 5}, "minecraft:wearable": {"slot": "slot.armor.head", "protection": 5}"#,
        &allowlisted(),
    );
    assert!(
        !found
            .iter()
            .any(|f| f.starts_with("core/dropped_field Info")),
        "{found:#?}"
    );
}

#[test]
fn a_component_with_one_member_of_another_type_refuses_the_item_in_the_legacy_chain_too() {
    // The legacy
    // chain's cereal slices (`ComponentItemDataLoader::load` → `_parseCereal`,
    // mask 0xfbf8) fail a member of another type (0x8) like the strict one; `load` then
    // takes `makeError` and `_loadItemDocument` logs
    // " Error Parsing Item" (`_contentLog(1, 3, 16)`): the item is refused.
    let d = rejects(
        &item(
            "1.20.0",
            "demo:x",
            r#""minecraft:durability": {"max_durability": 9, "damage_chance": {"min": "a"}}, "minecraft:max_stack_size": 3"#,
        ),
        &third_party(),
    );
    assert!(
        d.iter().any(|f| f.code.id() == "core/invalid_type"
            && f.severity == bedrock_addon_core::diagnostic::Severity::Error
            && f.location.json_path.to_string()
                == "minecraft:item/components/minecraft:durability/damage_chance"),
        "{d:#?}"
    );
    assert!(
        !d.iter().any(|f| f.code.kind()
            == bedrock_addon_core::diagnostic::Kind::UnknownMember(
                bedrock_addon_core::diagnostic::Disposition::Dropped
            )),
        "{d:#?}"
    );
}

#[test]
fn a_member_out_of_its_range_refuses_the_item_at_every_version() {
    // The component's own checks (a chance above 100, a weight limit out of range) were a
    // drop with a warning. The bindings' `BasicNumericConstraint`s are cereal's 0x40, an
    // error under both masks, after which the item is refused with " Error Parsing Item": a
    // rejecting error at the member, in the legacy chain as in the strict slices.
    // (The 1.21.30 `storage_item`'s bounds are not enforced by the game: a check of ours, the
    // component dropped with an information; the row is below.)
    for (version, components, member, component) in [
        (
            "1.26.30",
            r#""minecraft:durability": {"max_durability": 10, "damage_chance": 123457}, "minecraft:max_stack_size": 3"#,
            "components/minecraft:durability/damage_chance",
            "components/minecraft:durability",
        ),
        (
            "1.20.50",
            r#""minecraft:use_modifiers": {"use_duration": 1.6, "movement_modifier": 7}, "minecraft:max_stack_size": 3"#,
            "components/minecraft:use_modifiers/movement_modifier",
            "components/minecraft:use_modifiers",
        ),
    ] {
        let d = rejects(&item(version, "demo:x", components), &third_party());
        assert!(
            d.iter().any(|f| f.code.id() == "core/invalid_value"
                && f.severity == bedrock_addon_core::diagnostic::Severity::Error
                && f.location.json_path.to_string() == format!("minecraft:item/{member}")),
            "{version}: {d:#?}"
        );
        assert!(
            !d.iter().any(|f| f.code.id() == "core/dropped_component"
                && f.location.json_path.to_string() == format!("minecraft:item/{component}")),
            "{version}: {d:#?}"
        );
    }
    let (found, written) = told(
        "1.21.50",
        r#""minecraft:storage_item": {"max_slots": 2, "max_weight_limit": 123457}, "minecraft:max_stack_size": 3"#,
        &third_party(),
    );
    has(
        &found,
        "core/invalid_value Info at components/minecraft:storage_item/max_weight_limit",
    );
    assert!(!written.contains("storage_item"), "{written}");
}

/// The item of `text`, written, read again and written again: the second text.
#[track_caller]
fn rewritten(written: &str, cx: &LoadContext) -> String {
    let again = loads(written, cx);
    again
        .to_json()
        .expect("the written item writes again")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn a_negative_legacy_use_duration_is_told_at_the_member_of_the_file_and_not_carried() {
    // The note is not at `use_modifiers/use_duration`, a member
    // the file does not have, about the step's own default. The negative duration is told
    // where the file has it; nothing is told of the modifiers the step made.
    let (found, written) = told(
        "1.20.40",
        r#""minecraft:use_duration": -5, "minecraft:chargeable": {"movement_modifier": 0.5}"#,
        &allowlisted(),
    );
    // The bound is the newest format's; no constraint of the 1.20.40 slice's `use_duration`
    // was read: an information of ours.
    assert!(
        found
            .iter()
            .any(|f| f.starts_with("core/invalid_value Info at components/minecraft:use_duration")),
        "{found:#?}"
    );
    assert!(
        !found.iter().any(|f| f.contains("minecraft:use_modifiers")),
        "{found:#?}"
    );
    assert!(written.contains(r#""use_duration": 0.0"#), "{written}");
    assert!(written.contains(r#""movement_modifier": 0.5"#), "{written}");
    // (b): what is written reads back as it was written.
    assert_eq!(rewritten(&written, &allowlisted()), written);
}

#[test]
fn a_negative_legacy_use_duration_of_the_legacy_read_alone_is_told_and_gives_no_modifiers() {
    // From 1.20.50 the legacy read alone takes `minecraft:use_duration`. `min(0)` is the
    // newest format's (`use_duration: f32`, `min(0)`): a negative
    // duration has no text there, so it is told and not carried.
    for components in [
        r#""minecraft:use_duration": -5, "minecraft:max_stack_size": 3"#,
        r#""minecraft:use_duration": {"value": -5}, "minecraft:max_stack_size": 3"#,
    ] {
        let (found, written) = told("1.21.0", components, &third_party());
        assert!(
            found
                .iter()
                .any(|f| f
                    .starts_with("core/invalid_value Info at components/minecraft:use_duration")),
            "{found:#?}"
        );
        assert!(!written.contains("use_modifiers"), "{written}");
        assert_eq!(rewritten(&written, &third_party()), written);
    }
    // The file's own modifiers keep their duration.
    let (found, written) = told(
        "1.21.0",
        r#""minecraft:use_duration": -5, "minecraft:use_modifiers": {"use_duration": 2.0}"#,
        &third_party(),
    );
    assert!(
        !found.iter().any(|f| f.starts_with("core/dropped_field")),
        "{found:#?}"
    );
    assert!(written.contains(r#""use_duration": 2.0"#), "{written}");
}

/// The findings and the written text of a whole item payload (the members of
/// `minecraft:item` after the description) at `version`.
#[track_caller]
fn told_of_payload(version: &str, rest: &str, cx: &LoadContext) -> (Vec<String>, String) {
    let text = |rest: &str| {
        format!(
            r#"{{"format_version": "{version}", "minecraft:item": {{"description": {{"identifier": "demo:x"}}, {rest}}}}}"#
        )
    };
    let first = super::common::load(&text(rest), cx);
    // A side-car of 1.20.30 or later is a deprecated name the game refuses: the
    // findings are the rejected ones and there is no text.
    let mut document = match first {
        Ok(document) => document,
        Err(rejected) => {
            let found = rejected
                .diagnostics
                .iter()
                .map(|f| {
                    let code: String = f.code.id().to_owned();
                    let at = f.location.json_path.to_string();
                    let at = at.strip_prefix("minecraft:item/").unwrap_or(&at).to_owned();
                    format!("{code} {:?} at {at}", f.severity)
                })
                .collect();
            return (found, String::new());
        }
    };
    if refused_at_components(&document) {
        let rest = rest.replacen(
            r#""components": {"#,
            r#""components": {"minecraft:max_stack_size": 16, "#,
            1,
        );
        document = loads(&text(&rest), cx);
    }
    let found = document
        .diagnostics()
        .iter()
        .map(|f| {
            let code: String = f.code.id().to_owned();
            let at = f.location.json_path.to_string();
            let at = at.strip_prefix("minecraft:item/").unwrap_or(&at).to_owned();
            format!("{code} {:?} at {at}", f.severity)
        })
        .collect();
    let written = written_or_refused(&document);
    (
        found,
        written.split_whitespace().collect::<Vec<_>>().join(" "),
    )
}

#[test]
fn the_side_car_of_the_file_a_step_writes_over_or_does_not_read_is_told() {
    // Below 1.20.30 no struct binds `__deprecated_components`: the game does not read it,
    // and the written item has one member of that name, the steps'. Noted and not held.
    let (found, written) = told_of_payload(
        "1.20.20",
        r#""components": {"minecraft:food": {"nutrition": 4, "on_consume": {"event": "e"}}}, "__deprecated_components": {"on_consume_trigger": {"on_consume_trigger": {"event": "x:own"}}}"#,
        &allowlisted(),
    );
    has(&found, "core/dropped_field Info at __deprecated_components");
    assert!(written.contains(r#""event": "e""#), "{written}");
    assert!(!written.contains("x:own"), "{written}");
    // From 1.20.30 the side-car is read: a slot a step fills over what the file holds there
    // is told at the slot, and a member beside a slot's trigger at the member.
    let (found, written) = told_of_payload(
        "1.20.30",
        r#""components": {"minecraft:on_use": {"on_use": {"event": "e"}}}, "__deprecated_components": {"on_use": {"on_use_trigger": {"event": "x:own"}, "zz": 3}}"#,
        &allowlisted(),
    );
    // (the side-car is a deprecated name of the 1.20.30 root, which the game refuses, so no
    // step runs and nothing is written:)
    has(
        &found,
        "item/deprecated_components_deprecated Error at __deprecated_components",
    );
    has(
        &found,
        "core/member_not_kept Info at __deprecated_components/on_use/zz",
    );
    assert!(written.is_empty(), "{written}");
    // The same trigger: nothing is replaced.
    let (found, _) = told_of_payload(
        "1.20.30",
        r#""components": {"minecraft:on_use": {"on_use": {"event": "e"}}}, "__deprecated_components": {"on_use": {"on_use_trigger": {"event": "e"}}}"#,
        &allowlisted(),
    );
    has(
        &found,
        "item/deprecated_components_deprecated Error at __deprecated_components",
    );
}

#[test]
fn the_members_of_an_armor_the_1_20_20_step_removes_and_does_not_read_are_told() {
    let (found, _) = told(
        "1.20.0",
        r#""minecraft:armor": {"protection": 3, "zz": 3}, "minecraft:wearable": {"slot": "slot.armor.head"}"#,
        &allowlisted(),
    );
    has(
        &found,
        "core/unrecognized_field Warning at components/minecraft:armor/zz",
    );
}

#[test]
fn a_deprecated_on_dig_of_a_legacy_digger_the_1_20_50_step_writes_over_is_told() {
    // `__deprecated_on_dig` is the member the 1.20.50 step moves `on_dig` to; below it no
    // struct reads the name, and the step's own takes its place.
    let (found, written) = told(
        "1.20.40",
        r#""minecraft:digger": {"use_efficiency": true, "on_dig": {"event": "x:dug"}, "__deprecated_on_dig": {"event": "x:own"}, "destroy_speeds": [{"block": "minecraft:dirt", "speed": 6, "on_dig": {"event": "x:dirt"}, "__deprecated_on_dig": {"event": "x:own"}}]}"#,
        &allowlisted(),
    );
    has(
        &found,
        "core/dropped_field Info at components/minecraft:digger/__deprecated_on_dig",
    );
    has(
        &found,
        "core/dropped_field Info at components/minecraft:digger/destroy_speeds/0/__deprecated_on_dig",
    );
    assert!(!written.contains("x:own"), "{written}");
    assert!(
        written.contains("x:dug") && written.contains("x:dirt"),
        "{written}"
    );
}

#[test]
fn what_a_typed_read_does_not_hold_is_told_where_the_file_has_it() {
    // A member of a range given as an object that the range does not read, a member of a
    // custom component entry beside `name` and `data`: neither has a place in what is
    // held, so each is told (a component's own unknown member is kept, and says so).
    // Cereal's 0x4 (`IntRange::bindType` binds `min` and `max` alone): a level-2
    // line under the legacy chain's 0xfbf8, an error that refuses the item in a
    // strict slice.
    let d = rejects(
        &item(
            "1.26.30",
            "demo:x",
            r#""minecraft:durability": {"max_durability": 10, "damage_chance": {"min": 10, "max": 50, "zz": 3}}"#,
        ),
        &allowlisted(),
    );
    assert!(
        d.iter().any(|f| f.code.id() == "core/member_not_kept"
            && f.severity == bedrock_addon_core::diagnostic::Severity::Error
            && f.location.json_path.to_string()
                == "minecraft:item/components/minecraft:durability/damage_chance/zz"),
        "{d:#?}"
    );
    for (version, components, at) in [
        (
            "1.16.100",
            r#""minecraft:durability": {"max_durability": 10, "damage_chance": {"min": 10, "max": 50, "zz": 3}}"#,
            "components/minecraft:durability/damage_chance/zz",
        ),
        (
            "1.21.60",
            r#""minecraft:custom_components": [{"name": "demo:shake", "data": {"a": 1}, "zz": 3}]"#,
            "components/minecraft:custom_components/0/zz",
        ),
    ] {
        let (found, written) = told(version, components, &allowlisted());
        has(&found, &format!("core/member_not_kept Warning at {at}"));
        assert!(!written.contains("zz"), "{version}: {written}");
    }
}

#[test]
fn a_storage_weight_component_of_the_file_the_1_21_60_split_writes_over_is_noted() {
    // No slice binds `storage_weight_limit` or `storage_weight_modifier` at 1.21.50: one the
    // file has is no component to the game, and the split's own takes its place.
    let (found, written) = told(
        "1.21.50",
        r#""minecraft:max_stack_size": 1, "minecraft:storage_item": {"max_slots": 2, "max_weight_limit": 32}, "minecraft:storage_weight_limit": {"max_weight_limit": 33}"#,
        &third_party(),
    );
    has(
        &found,
        "core/dropped_field Info at components/minecraft:storage_weight_limit",
    );
    assert!(written.contains(r#""max_weight_limit": 32"#), "{written}");
    assert!(!written.contains("33"), "{written}");
}

#[test]
fn a_movement_modifier_out_of_the_modifiers_range_is_told_and_not_carried() {
    // `movement_modifier: 7` of a
    // legacy `chargeable` was carried into `use_modifiers`, whose reader takes 0 to 1 and
    // drops the component for anything else: the written text did not reload to itself.
    // (With a component beside the two, the item keeps it; an item left without any has no text.)
    let (found, written) = told(
        "1.20.40",
        r#""minecraft:use_duration": -5, "minecraft:chargeable": {"movement_modifier": 7}, "minecraft:max_stack_size": 3"#,
        &allowlisted(),
    );
    has(
        &found,
        // ours, an information
        "core/invalid_value Info at components/minecraft:chargeable/movement_modifier",
    );
    assert!(!written.contains("movement_modifier"), "{written}");
    assert_eq!(rewritten(&written, &allowlisted()), written);
    // In the range it is carried, as before.
    let (found, written) = told(
        "1.20.40",
        r#""minecraft:chargeable": {"movement_modifier": 1}"#,
        &allowlisted(),
    );
    assert!(
        !found.iter().any(|f| f.starts_with("core/invalid_value")),
        "{found:#?}"
    );
    assert!(written.contains(r#""movement_modifier": 1.0"#), "{written}");
    assert_eq!(rewritten(&written, &allowlisted()), written);
}

/// An item whose step fills
/// the side-car beside payload `events` holds both; neither is a member of the strict 1.26.50
/// format, so the newest writer refuses the item at the first of them, and writes
/// nothing the game would refuse.
#[test]
fn an_item_with_a_side_car_beside_events_is_written_as_its_reload_writes_it() {
    let text = r#"{"format_version": "1.20.20", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:food": {"nutrition": 1, "on_consume": {"event": "a"}}, "minecraft:use_duration": 1.6}, "events": {"a": {}}}}"#;
    let document = loads(text, &allowlisted());
    let payload = newest_payload(&document);
    assert!(
        payload.contains("__deprecated_components") && payload.contains(r#""events""#),
        "the item holds both members: {payload}"
    );
    let written = document.to_json();
    assert!(
        super::common::refused_at(&written, "minecraft:item/events"),
        "{written:?}"
    );
}

/// The load says where the newest writer refuses such an item, with the note
/// `NOT_IN_THE_NEWEST_FORMAT`: ours, the game reads the older file and says
/// nothing, an information.
#[test]
fn the_member_the_newest_format_refuses_is_noted_at_the_load_as_an_information() {
    let text = r#"{"format_version": "1.20.20", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:food": {"nutrition": 1, "on_consume": {"event": "a"}}, "minecraft:use_duration": 1.6}, "events": {"a": {}}}}"#;
    let document = loads(text, &allowlisted());
    let note: Vec<(String, bedrock_addon_core::diagnostic::Severity)> = document
        .diagnostics()
        .iter()
        .filter(|f| f.to_string().ends_with(
            "the newest format (1.26.50) does not bind this member and refuses a document with it: the item is written as a 1.26.0 document when that text reads back as the item, else not written",
        ))
        .map(|f| (f.location.json_path.to_string(), f.severity))
        .collect();
    assert_eq!(
        note,
        [(
            "minecraft:item/events".to_owned(),
            bedrock_addon_core::diagnostic::Severity::Info
        )]
    );
}

/// In a cereal item (1.26.30 and later, the
/// `MinecraftDocumentInput` slices) a component whose typed read fails (`0x8`) or whose required
/// member is missing (`0x80`) rejects the item with an error at the member, under both masks;
/// below 1.26.30 (1.21.0: the `JsonCpp` `ComponentItemData_*` chain) the read fails the same way
/// and the item is refused as well.
#[test]
fn a_failed_component_read_rejects_the_item_in_a_cereal_and_in_a_legacy_one() {
    let cases = [
        (
            r#""minecraft:durability": {"max_durability": 9, "damage_chance": {"min": "a"}}, "minecraft:max_stack_size": 3"#,
            "core/invalid_type",
            "minecraft:item/components/minecraft:durability/damage_chance",
        ),
        (
            r#""minecraft:cooldown": {"duration": 1.0}, "minecraft:max_stack_size": 3"#,
            "core/missing_required_field",
            "minecraft:item/components/minecraft:cooldown/category",
        ),
    ];
    let name =
        |d: &bedrock_addon_core::diagnostic::Diagnostic| -> String { d.code.id().to_owned() };
    for (components, code, at) in cases {
        for version in ["1.21.0", "1.26.30", "1.26.50"] {
            let d = rejects(&item(version, "demo:x", components), &third_party());
            let errors: Vec<(String, String)> = d
                .iter()
                .filter(|f| f.severity == bedrock_addon_core::diagnostic::Severity::Error)
                .map(|f| (name(f), f.location.json_path.to_string()))
                .collect();
            assert_eq!(
                errors,
                [(code.to_owned(), at.to_owned())],
                "{version} {components}"
            );
        }
        // The legacy chain too: its cereal slices fail the read the same way
        // (0x8 / 0x80 under 0xfbf8) and `makeError` refuses the item.
        let d = rejects(&item("1.21.0", "demo:x", components), &third_party());
        let errors: Vec<(String, String)> = d
            .iter()
            .filter(|f| f.severity == bedrock_addon_core::diagnostic::Severity::Error)
            .map(|f| (name(f), f.location.json_path.to_string()))
            .collect();
        assert_eq!(
            errors,
            [(code.to_owned(), at.to_owned())],
            "1.21.0 {components}"
        );
    }
}
