//! The audit of the legacy reading of a component-based item, over every format version and
//! through both writers: the newest one (the item upgraded) and the one of the file's own
//! struct (the item as it was read). Every member the legacy reader takes is in the written
//! text, or the load says at the member that it is not, with the code and severity pinned.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::document::JsonDocument;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_item::Item;

use super::common::*;

/// Every format version with a struct of its own, oldest first.
const VERSIONS: &[&str] = &[
    "1.16.100", "1.19.83", "1.20.0", "1.20.10", "1.20.20", "1.20.30", "1.20.40", "1.20.50",
    "1.20.60", "1.20.80", "1.21.10", "1.21.30", "1.21.40", "1.21.50", "1.21.60", "1.21.80",
    "1.21.90", "1.21.110", "1.26.0", "1.26.30",
];

/// The members the legacy reader takes: the field of `LegacyItemData` it fills, where it is
/// (`description:` or a key of `components`), its JSON, and a text that is in a written
/// document exactly when the value is.
const MEMBERS: &[(&str, &str, &str, &str)] = &[
    ("category", "description:category", r#""Nature""#, "Nature"),
    (
        "block",
        "minecraft:block",
        r#""minecraft:stone""#,
        "minecraft:stone",
    ),
    ("mining_speed", "minecraft:mining_speed", "2.625", "2.625"),
    ("frame_count", "minecraft:frame_count", "37", "37"),
    (
        "creative_category",
        "minecraft:creative_category",
        r#"{"parent": "zz.parent"}"#,
        "zz.parent",
    ),
    ("tags", "tag:zz_tag", "{}", "zz_tag"),
    ("is_glint", "minecraft:foil", "true", "minecraft:foil"),
    (
        "seed",
        "minecraft:seed",
        r#"{"crop_result": "zz:crop"}"#,
        "zz:crop",
    ),
    // (A whole number of ticks: the game holds the duration in ticks.)
    ("shared", "minecraft:use_duration", "1.65", "1.65"),
    ("shared", "minecraft:rarity", r#""epic""#, "epic"),
    (
        "shared",
        "minecraft:use_modifiers",
        r#"{"use_duration": 1.65, "movement_modifier": 0.375}"#,
        "0.375",
    ),
    (
        "shared",
        "minecraft:hover_text_color",
        r#""light_purple""#,
        "light_purple",
    ),
    (
        "shared",
        "minecraft:use_animation",
        r#""crossbow""#,
        "crossbow",
    ),
    (
        "shared",
        "minecraft:enchantable",
        r#"{"slot": "bow", "value": 13}"#,
        "13",
    ),
    (
        "shared",
        "minecraft:interact_button",
        r#""zz button""#,
        "zz button",
    ),
    ("shared", "minecraft:damage", "13", "13"),
    (
        "shared",
        "minecraft:stacked_by_data",
        "true",
        "stacked_by_data",
    ),
    (
        "shared",
        "minecraft:should_despawn",
        "false",
        "should_despawn",
    ),
    (
        "shared",
        "minecraft:liquid_clipped",
        "true",
        "liquid_clipped",
    ),
    ("shared", "minecraft:hand_equipped", "true", "hand_equipped"),
    // (With its quotes: the key, which `is_glint` of every item's `Debug` is not.)
    ("shared", "minecraft:glint", "true", "\"minecraft:glint\""),
    (
        "shared",
        "minecraft:can_destroy_in_creative",
        "false",
        "can_destroy_in_creative",
    ),
    (
        "shared",
        "minecraft:allow_off_hand",
        "true",
        "allow_off_hand",
    ),
];

fn document(version: &str, at: &str, json: &str) -> String {
    let (description, components) = match at.strip_prefix("description:") {
        Some(member) => (format!(r#", "{member}": {json}"#), String::new()),
        None => (String::new(), format!(r#""{at}": {json}, "#)),
    };
    format!(
        r#"{{"format_version": "{version}", "minecraft:item": {{"description": {{"identifier": "demo:x"{description}}}, "components": {{{components}"minecraft:max_stack_size": 16, "minecraft:icon": "zz_icon"}}}}}}"#
    )
}

/// What becomes of the member: `written` (the text has it, and the text read back by
/// `reread` holds it), or what the load tells at it, or `LOST`.
fn outcome(
    diagnostics: &bedrock_addon_core::diagnostic::Diagnostics,
    written: Result<String, bedrock_addon_core::json::EncodeError>,
    reread: &dyn Fn(&str) -> Option<String>,
    at: &str,
    token: &str,
) -> String {
    let member = at.strip_prefix("description:").unwrap_or(at);
    let told: Vec<String> = diagnostics
        .iter()
        .filter(|f| {
            f.location.json_path.segments().any(|segment| {
                matches!(segment, bedrock_addon_core::diagnostic::JsonPathSegment::Key(key) if key == member)
            })
        })
        .map(|f| format!("{} {:?}", f.code.id(), f.severity))
        .collect();
    match written {
        Ok(text) if text.contains(token) => {
            // In the text is not enough: the reader of that text must hold it.
            match reread(&text) {
                // (`foil` is held as the flag it sets, not under its key.)
                Some(held)
                    if held.contains(token)
                        || (token == "minecraft:foil" && held.contains("is_glint: Some(true)")) => {
                }
                Some(_) => return "LOST (written where no reader takes it)".to_owned(),
                None => return "LOST (the written text does not load)".to_owned(),
            }
            if told.is_empty() {
                "written".to_owned()
            } else {
                format!("written, told {}", told.join(" + "))
            }
        }
        Ok(_) if !told.is_empty() => format!("told {}", told.join(" + ")),
        Ok(_) => "LOST".to_owned(),
        Err(_) => "refused".to_owned(),
    }
}

/// The outcomes of one member over the versions in the context `cx`, as bands:
/// `from version: newest | as read`.
fn bands(
    cx: &bedrock_addon_core::load::LoadContext,
    at: &str,
    json: &str,
    token: &str,
) -> Vec<String> {
    let path = PackPath::new("items/x.json").unwrap();
    let newest_again = |text: &str| load(text, cx).ok().map(|d| format!("{:?}", d.value()));
    let as_read_again = |text: &str| {
        Item::parse_as_written(&path, text.as_bytes(), cx)
            .ok()
            .map(|d| format!("{:?}", d.value()))
    };
    let mut out: Vec<String> = Vec::new();
    let mut last = String::new();
    for version in VERSIONS {
        let text = document(version, at, json);
        let newest = match load(&text, cx) {
            Ok(document) => outcome(
                document.diagnostics(),
                document.to_json(),
                &newest_again,
                at,
                token,
            ),
            Err(_) => "rejected".to_owned(),
        };
        let as_read = match Item::parse_as_written(&path, text.as_bytes(), cx) {
            Ok(document) => outcome(
                document.diagnostics(),
                document.to_json(),
                &as_read_again,
                at,
                token,
            ),
            Err(_) => "rejected".to_owned(),
        };
        let now = format!("{newest} | {as_read}");
        if now != last {
            out.push(format!("{version}: {now}"));
            last = now;
        }
    }
    out
}

/// The fields of `LegacyItemData` that are no row of the table: `format` and `format_item`
/// are what a document of the legacy item format fills (it has no components to audit by
/// version; `a_legacy_format_item_is_written_whole_by_both_writers` holds them), the two
/// marks are set by the load, and `expressions` is the compiled Molang of members that are
/// written where they are.
const DERIVED: &[&str] = &[
    "format",
    "format_item",
    "left_empty_by_the_load",
    "expressions",
    "held_in_shared_alone",
];

/// The names of the fields of `LegacyItemData`, from its `Debug`.
fn fields() -> Vec<String> {
    let debug = format!("{:?}", bedrock_addon_family_item::LegacyItemData::default());
    let body = debug
        .strip_prefix("LegacyItemData { ")
        .and_then(|rest| rest.strip_suffix(" }"))
        .expect("the Debug of a struct");
    let mut fields = Vec::new();
    let (mut depth, mut start) = (0usize, 0);
    for (index, c) in body.char_indices() {
        match c {
            '{' | '[' | '(' => depth += 1,
            '}' | ']' | ')' => depth -= 1,
            ':' if depth == 0 && body[index..].starts_with(": ") => {
                if let Some(name) = body[start..index].rsplit(", ").next() {
                    fields.push(name.to_owned());
                }
                start = index;
            }
            _ => {}
        }
    }
    fields
}

#[test]
fn every_member_of_the_legacy_reading_is_written_or_told_by_both_writers_at_every_version() {
    // Every field of the struct is a row of this table or is named as derived: a field
    // added later fails here until it is one or the other.
    let mut covered: Vec<&str> = MEMBERS
        .iter()
        .map(|row| row.0)
        .chain(DERIVED.iter().copied())
        .collect();
    covered.sort_unstable();
    covered.dedup();
    let mut of_the_struct = fields();
    of_the_struct.sort();
    assert_eq!(
        covered, of_the_struct,
        "a row or a reason for every field of `LegacyItemData`"
    );
    let mut lost = Vec::new();
    let mut found = Vec::new();
    for (context, cx) in [
        ("allow-listed", allowlisted()),
        ("third-party", third_party()),
    ] {
        for (field, at, json, token) in MEMBERS {
            for band in bands(&cx, at, json, token) {
                let line = format!("{context}: {field} `{at}` {band}");
                if line.contains("LOST") {
                    lost.push(line.clone());
                }
                found.push(line);
            }
        }
    }
    for line in &found {
        println!("    {line:?},");
    }
    assert!(lost.is_empty(), "lost in silence:\n{}", lost.join("\n"));
    let found: Vec<&str> = found.iter().map(String::as_str).collect();
    assert_eq!(found, EXPECTED);
}

/// What becomes of each member in each context, by the version the outcome starts at:
/// `newest | as read`. (A third-party component-based item below 1.20.0 is rejected.)
const EXPECTED: &[&str] = &[
    "allow-listed: category `description:category` 1.16.100: told core/invalid_value Info | written",
    "allow-listed: category `description:category` 1.20.20: told core/dropped_field Info | written",
    "allow-listed: category `description:category` 1.20.60: written, told core/unrecognized_field Warning + core/invalid_value Info | written, told core/unrecognized_field Warning",
    "allow-listed: category `description:category` 1.21.60: refused | written, told core/unrecognized_field Warning",
    "allow-listed: category `description:category` 1.21.110: written, told core/unrecognized_field Warning + core/invalid_value Info | written, told core/unrecognized_field Warning",
    "allow-listed: category `description:category` 1.26.30: rejected | rejected",
    "allow-listed: block `minecraft:block` 1.16.100: written | written",
    "allow-listed: mining_speed `minecraft:mining_speed` 1.16.100: told core/dropped_field Info | written",
    "allow-listed: mining_speed `minecraft:mining_speed` 1.20.30: told item/mining_speed_deprecated Error | told item/mining_speed_deprecated Error",
    "allow-listed: frame_count `minecraft:frame_count` 1.16.100: written | written",
    "allow-listed: creative_category `minecraft:creative_category` 1.16.100: told core/dropped_field Info | written",
    "allow-listed: creative_category `minecraft:creative_category` 1.20.20: told item/creative_category_deprecated Error | told item/creative_category_deprecated Error",
    "allow-listed: tags `tag:zz_tag` 1.16.100: written | written",
    "allow-listed: tags `tag:zz_tag` 1.20.50: told item/tag_deprecated Error | told item/tag_deprecated Error",
    "allow-listed: is_glint `minecraft:foil` 1.16.100: told core/dropped_field Info | written",
    "allow-listed: is_glint `minecraft:foil` 1.20.20: told item/foil_deprecated Error | told item/foil_deprecated Error",
    "allow-listed: seed `minecraft:seed` 1.16.100: told core/dropped_field Info | told core/dropped_field Info",
    "allow-listed: seed `minecraft:seed` 1.19.83: told item/seed_deprecated Warning | told item/seed_deprecated Warning",
    "allow-listed: shared `minecraft:use_duration` 1.16.100: written | written",
    "allow-listed: shared `minecraft:rarity` 1.16.100: written | written",
    "allow-listed: shared `minecraft:use_modifiers` 1.16.100: told core/dropped_field Info | written, told core/dropped_field Info",
    "allow-listed: shared `minecraft:use_modifiers` 1.20.50: written | written",
    "allow-listed: shared `minecraft:hover_text_color` 1.16.100: written | written",
    "allow-listed: shared `minecraft:use_animation` 1.16.100: written | written",
    "allow-listed: shared `minecraft:enchantable` 1.16.100: written | written",
    "allow-listed: shared `minecraft:interact_button` 1.16.100: written | written",
    "allow-listed: shared `minecraft:damage` 1.16.100: written | written",
    "allow-listed: shared `minecraft:stacked_by_data` 1.16.100: written | written",
    "allow-listed: shared `minecraft:should_despawn` 1.16.100: written | written",
    "allow-listed: shared `minecraft:liquid_clipped` 1.16.100: written | written",
    "allow-listed: shared `minecraft:hand_equipped` 1.16.100: written | written",
    "allow-listed: shared `minecraft:glint` 1.16.100: written | written",
    "allow-listed: shared `minecraft:can_destroy_in_creative` 1.16.100: written | written",
    "allow-listed: shared `minecraft:allow_off_hand` 1.16.100: written | written",
    "third-party: category `description:category` 1.16.100: rejected | rejected",
    "third-party: category `description:category` 1.20.0: told core/invalid_value Info | written",
    "third-party: category `description:category` 1.20.20: told core/dropped_field Info | written",
    "third-party: category `description:category` 1.20.60: written, told core/unrecognized_field Warning + core/invalid_value Info | written, told core/unrecognized_field Warning",
    "third-party: category `description:category` 1.21.60: refused | written, told core/unrecognized_field Warning",
    "third-party: category `description:category` 1.21.110: written, told core/unrecognized_field Warning + core/invalid_value Info | written, told core/unrecognized_field Warning",
    "third-party: category `description:category` 1.26.30: rejected | rejected",
    "third-party: block `minecraft:block` 1.16.100: rejected | rejected",
    "third-party: block `minecraft:block` 1.20.0: written | written",
    "third-party: mining_speed `minecraft:mining_speed` 1.16.100: rejected | rejected",
    "third-party: mining_speed `minecraft:mining_speed` 1.20.0: told core/dropped_field Info | written",
    "third-party: mining_speed `minecraft:mining_speed` 1.20.30: told item/mining_speed_deprecated Error | told item/mining_speed_deprecated Error",
    "third-party: frame_count `minecraft:frame_count` 1.16.100: rejected | rejected",
    "third-party: frame_count `minecraft:frame_count` 1.20.0: written | written",
    "third-party: creative_category `minecraft:creative_category` 1.16.100: rejected | rejected",
    "third-party: creative_category `minecraft:creative_category` 1.20.0: told core/dropped_field Info | written",
    "third-party: creative_category `minecraft:creative_category` 1.20.20: told item/creative_category_deprecated Error | told item/creative_category_deprecated Error",
    "third-party: tags `tag:zz_tag` 1.16.100: rejected | rejected",
    "third-party: tags `tag:zz_tag` 1.20.0: written | written",
    "third-party: tags `tag:zz_tag` 1.20.50: told item/tag_deprecated Error | told item/tag_deprecated Error",
    "third-party: is_glint `minecraft:foil` 1.16.100: rejected | rejected",
    "third-party: is_glint `minecraft:foil` 1.20.0: told item/foil_ignored Info | told item/foil_ignored Info",
    "third-party: is_glint `minecraft:foil` 1.20.20: told item/foil_deprecated Error | told item/foil_deprecated Error",
    "third-party: seed `minecraft:seed` 1.16.100: rejected | rejected",
    "third-party: seed `minecraft:seed` 1.20.0: told item/seed_deprecated Warning | told item/seed_deprecated Warning",
    "third-party: shared `minecraft:use_duration` 1.16.100: rejected | rejected",
    "third-party: shared `minecraft:use_duration` 1.20.0: written | written",
    "third-party: shared `minecraft:rarity` 1.16.100: rejected | rejected",
    "third-party: shared `minecraft:rarity` 1.20.0: written | written",
    "third-party: shared `minecraft:use_modifiers` 1.16.100: rejected | rejected",
    "third-party: shared `minecraft:use_modifiers` 1.20.0: told core/dropped_field Info | written, told core/dropped_field Info",
    "third-party: shared `minecraft:use_modifiers` 1.20.50: written | written",
    "third-party: shared `minecraft:hover_text_color` 1.16.100: rejected | rejected",
    "third-party: shared `minecraft:hover_text_color` 1.20.0: written | written",
    "third-party: shared `minecraft:use_animation` 1.16.100: rejected | rejected",
    "third-party: shared `minecraft:use_animation` 1.20.0: written | written",
    "third-party: shared `minecraft:enchantable` 1.16.100: rejected | rejected",
    "third-party: shared `minecraft:enchantable` 1.20.0: written | written",
    "third-party: shared `minecraft:interact_button` 1.16.100: rejected | rejected",
    "third-party: shared `minecraft:interact_button` 1.20.0: written | written",
    "third-party: shared `minecraft:damage` 1.16.100: rejected | rejected",
    "third-party: shared `minecraft:damage` 1.20.0: written | written",
    "third-party: shared `minecraft:stacked_by_data` 1.16.100: rejected | rejected",
    "third-party: shared `minecraft:stacked_by_data` 1.20.0: written | written",
    "third-party: shared `minecraft:should_despawn` 1.16.100: rejected | rejected",
    "third-party: shared `minecraft:should_despawn` 1.20.0: written | written",
    "third-party: shared `minecraft:liquid_clipped` 1.16.100: rejected | rejected",
    "third-party: shared `minecraft:liquid_clipped` 1.20.0: written | written",
    "third-party: shared `minecraft:hand_equipped` 1.16.100: rejected | rejected",
    "third-party: shared `minecraft:hand_equipped` 1.20.0: written | written",
    "third-party: shared `minecraft:glint` 1.16.100: rejected | rejected",
    "third-party: shared `minecraft:glint` 1.20.0: written | written",
    "third-party: shared `minecraft:can_destroy_in_creative` 1.16.100: rejected | rejected",
    "third-party: shared `minecraft:can_destroy_in_creative` 1.20.0: written | written",
    "third-party: shared `minecraft:allow_off_hand` 1.16.100: rejected | rejected",
    "third-party: shared `minecraft:allow_off_hand` 1.20.0: written | written",
];

/// The writer of a file's own struct writes only what that format has: a `use_modifiers`
/// below 1.26.30 has no `start_using`. Its text reads with the findings the file had and
/// upgrades to the item the file upgrades to.
#[test]
fn an_item_written_as_it_was_read_has_only_the_members_of_its_format() {
    let cx = third_party();
    let path = PackPath::new("items/x.json").unwrap();
    let told = |d: &bedrock_addon_core::diagnostic::Diagnostics| -> Vec<String> {
        d.iter()
            .map(|f| {
                format!(
                    "{} {:?} at {}",
                    f.code.id(),
                    f.severity,
                    f.location.json_path
                )
            })
            .collect()
    };
    for version in VERSIONS
        .iter()
        .filter(|version| !["1.16.100", "1.19.83"].contains(version))
    {
        for member in [
            r#""minecraft:use_modifiers": {"use_duration": 1.65, "movement_modifier": 0.375}"#,
            r#""minecraft:use_duration": 1.65"#,
            r#""tag:zz_tag": {}"#,
            r#""minecraft:rarity": "epic""#,
        ] {
            let text = format!(
                r#"{{"format_version": "{version}", "minecraft:item": {{"description": {{"identifier": "demo:x"}}, "components": {{{member}, "minecraft:max_stack_size": 16}}}}}}"#
            );
            let Ok(as_read) = Item::parse_as_written(&path, text.as_bytes(), &cx) else {
                continue;
            };
            let written = as_read.to_json().expect("written as read");
            if *version != "1.26.30" {
                assert!(
                    !written.contains("start_using"),
                    "{version} {member}: {written}"
                );
            }
            // The text says what the file said, read as it is and upgraded.
            let again = Item::parse_as_written(&path, written.as_bytes(), &cx).expect("reads");
            // (Nothing new: a member told as not read is not in the text, nor its finding.)
            let subset = |of: Vec<String>, whole: Vec<String>| of.iter().all(|f| whole.contains(f));
            assert!(
                subset(told(again.diagnostics()), told(as_read.diagnostics())),
                "{version} {member}: {written}"
            );
            let (first, second) = (loads(&text, &cx), loads(&written, &cx));
            assert!(
                subset(told(second.diagnostics()), told(first.diagnostics())),
                "{version} {member}: {written}"
            );
            assert_eq!(
                second.value().components,
                first.value().components,
                "{version} {member}: {written}"
            );
            assert_eq!(
                second.to_json().unwrap(),
                first.to_json().unwrap(),
                "{version} {member}"
            );
        }
    }
}

/// A top-level `description.category` of a file from 1.20.20 on is no member of its format.
/// From 1.20.60 the format tells of it, and it is kept: the upgraded text has it and says
/// what the first load said. From 1.20.20 to 1.20.50 the format reads it without a word and
/// the game writes the document on without it: it is not carried into the newest format,
/// which would tell of it, and the upgrade notes that it goes. Either way the written text
/// says nothing the first load did not, and is written as itself.
// Coordinator ruling on the audit's find (round 3).
#[test]
fn a_top_level_category_is_kept_where_its_format_tells_of_it_and_noted_where_it_goes() {
    let cx = allowlisted();
    let at_category = |d: &bedrock_addon_core::diagnostic::Diagnostics| -> Vec<String> {
        d.iter()
            .filter(|f| {
                f.location.json_path.segments().last().is_some_and(|last| {
                    matches!(
                        last,
                        bedrock_addon_core::diagnostic::JsonPathSegment::Key("category")
                    )
                })
            })
            .filter(|f| f.location.json_path.segments().len() == 3)
            .map(|f| format!("{} {:?} {:?}", f.code.id(), f.severity, f.phase))
            .collect()
    };
    for version in VERSIONS
        .iter()
        .filter(|version| !["1.16.100", "1.19.83", "1.20.0", "1.20.10"].contains(version))
    {
        let text = document(version, "description:category", r#""Nature""#);
        if *version == "1.26.30" {
            // a modern slice: the unknown member is an error that rejects the item
            assert_eq!(
                at_category(&rejects(&text, &cx)),
                ["core/unrecognized_field Error Parse"],
                "{version}"
            );
            continue;
        }
        let first = loads(&text, &cx);
        let silent = ["1.20.20", "1.20.30", "1.20.40", "1.20.50"].contains(version);
        if silent {
            let written = first.to_json().expect("written");
            let second = loads(&written, &cx);
            assert_eq!(
                at_category(first.diagnostics()),
                ["core/dropped_field Info Upgrade"],
                "{version}"
            );
            assert!(!written.contains("Nature"), "{version}: {written}");
            assert_eq!(
                at_category(second.diagnostics()),
                [] as [&str; 0],
                "{version}"
            );
            assert_eq!(
                second.to_json().expect("written again"),
                written,
                "{version}"
            );
        } else {
            assert_eq!(
                at_category(first.diagnostics()),
                [
                    "core/unrecognized_field Warning Parse",
                    "core/invalid_value Info Validate"
                ],
                "{version}"
            );
            // the 1.26.50 format refuses the member it does not know: the
            // newest writer writes the item at 1.26.0 where that reads back as the item, else
            // it refuses it at the member
            let written = first.to_json();
            assert!(
                refused_at(&written, "minecraft:item/description/category")
                    || written_at_1_26_0(&written).contains("Nature"),
                "{version}: {written:?}"
            );
            assert!(written_as_read(&text, &cx).contains("Nature"), "{version}");
        }
    }
}

/// What the load of a `minecraft:rarity` of `value` at `version` says of that member, sorted.
fn rarity_told(version: &str, value: &str) -> Vec<String> {
    let cx = third_party();
    let text = format!(
        r#"{{"format_version": "{version}", "minecraft:item": {{"description": {{"identifier": "demo:x", "menu_category": {{"category": "items"}}}}, "components": {{"minecraft:rarity": {value}, "minecraft:max_stack_size": 16}}}}}}"#
    );
    let mut told: Vec<String> = findings(&text, &cx)
        .iter()
        .filter(|f| {
            f.location.json_path.segments().any(|segment| {
                matches!(
                    segment,
                    bedrock_addon_core::diagnostic::JsonPathSegment::Key("minecraft:rarity")
                )
            })
        })
        .map(|f| {
            format!(
                "{} {:?} at {}",
                f.code.id(),
                f.severity,
                f.location.json_path
            )
        })
        .collect();
    told.sort();
    told
}

/// The `minecraft:rarity` member as the item of `version` holding `value` is written, or
/// `refused` or `absent`.
fn rarity_written(version: &str, value: &str) -> String {
    let cx = third_party();
    let text = format!(
        r#"{{"format_version": "{version}", "minecraft:item": {{"description": {{"identifier": "demo:x", "menu_category": {{"category": "items"}}}}, "components": {{"minecraft:rarity": {value}, "minecraft:max_stack_size": 16}}}}}}"#
    );
    // The member as it is written (the step puts it after the components the file's
    // format binds; the order of the members is nothing to the game).
    // A value the typed read fails refuses the item: nothing is written.
    let Ok(document) = load(&text, &cx) else {
        return "refused".to_owned();
    };
    let written = document.to_json().expect("written");
    let value: bedrock_addon_core::json::Value =
        bedrock_addon_core::json::from_str(&written).unwrap();
    let rarity = value.as_object().and_then(|root| {
        root.get("minecraft:item")?
            .as_object()?
            .get("components")?
            .as_object()?
            .get("minecraft:rarity")
            .cloned()
    });
    rarity.map_or_else(
        || "absent".to_owned(),
        |rarity| bedrock_addon_core::json::to_json(&rarity).unwrap(),
    )
}

/// The `minecraft:rarity` of a file from 1.20.30 to 1.21.29 is read into the component by
/// the 1.21.30 step, and what that reading says of the value is said: the upgraded load has
/// the findings a file of 1.21.30 with the same value has, at the same key, and a value the
/// component does not read at all is told as dropped.
#[test]
fn the_rarity_an_older_file_holds_is_read_with_what_the_1_21_30_reader_says_of_it() {
    let cx = third_party();
    for value in [
        r#""epic""#,
        "5",
        r#"{"value": 7, "zz": 1}"#,
        r#"{"value": "rare", "zz": 1}"#,
    ] {
        // The 1.21.30 slice's typed read fails a value of another type (`0x8`) and refuses
        // the item; below it the game's legacy reader takes the key in silence
        // and the same word is the crate's own information.
        let newer: Vec<String> = rarity_told("1.21.30", value)
            .iter()
            .map(|f| f.replace(" Error ", " Info "))
            .collect();
        // The refused item stops reading at the failed member; the legacy chain goes on and
        // tells what follows it too, so the older findings hold the newer ones.
        for version in ["1.20.30", "1.20.50", "1.21.10"] {
            let older = rarity_told(version, value);
            assert!(
                newer.iter().all(|f| older.contains(f)),
                "{version} {value}: {older:?} lacks {newer:?}"
            );
        }
    }
    assert_eq!(rarity_told("1.21.10", r#""epic""#), [] as [&str; 0]);
    assert!(!rarity_told("1.21.10", "5").is_empty(), "a number is told");
    // Whatever the value: the findings and the written text of the 1.21.30 file, exactly.
    for value in ["-0.0", "1.0", "null"] {
        // The 1.21.30 slice refuses the item for them: nothing is written. Below
        // it the legacy chain tells the same word as an information and writes the member.
        let newer: Vec<String> = rarity_told("1.21.30", value)
            .iter()
            .map(|f| f.replace(" Error ", " Info "))
            .collect();
        let kept = rarity_written("1.20.30", value);
        if value != "null" {
            assert_eq!(rarity_written("1.21.30", value), "refused", "{value}");
        }
        for version in ["1.20.30", "1.21.10"] {
            assert_eq!(
                (rarity_told(version, value), rarity_written(version, value)),
                (newer.clone(), kept.clone()),
                "{version} {value}"
            );
        }
    }
    // A value of another shape: at 1.21.30 the file's own slice reads it, cereal's 0x8, and
    // the item is refused; below it the game's legacy reader takes the key in
    // silence and the 1.21.30 reader's word is the crate's own (`EXPECTED_UNREAD`, below).
    for value in ["[1]", "true", r#"{"value": {"a": 1}}"#] {
        let text = format!(
            r#"{{"format_version": "1.21.30", "minecraft:item": {{"description": {{"identifier": "demo:x", "menu_category": {{"category": "items"}}}}, "components": {{"minecraft:rarity": {value}, "minecraft:max_stack_size": 16}}}}}}"#
        );
        let d = rejects(&text, &cx);
        assert!(
            d.iter().any(|f| f.severity == Severity::Error
                && f.location
                    .json_path
                    .to_string()
                    .starts_with("minecraft:item/components/minecraft:rarity")),
            "{value}: {d:#?}"
        );
    }
    // What a value that is no rarity is told and written as, at 1.21.30 and below it alike.
    for (value, findings, member) in EXPECTED_UNREAD {
        assert_eq!(rarity_told("1.21.10", value), *findings, "{value}");
        assert_eq!(
            rarity_written("1.21.10", value)
                .split_whitespace()
                .collect::<String>(),
            *member,
            "{value}"
        );
    }
}

/// An unknown member inside a `__deprecated_components.legacy_on_dig_trigger` a file below
/// 1.26.30 writes itself is kept with the digger it is read into and told where it is, as
/// an unknown member of a component is: the written text has it.
#[test]
fn an_unknown_member_of_a_side_car_digger_is_kept_and_told() {
    let cx = allowlisted();
    for version in ["1.20.30", "1.21.10", "1.26.0"] {
        let text = format!(
            r#"{{"format_version": "{version}", "minecraft:item": {{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:max_stack_size": 16}}, "__deprecated_components": {{"legacy_on_dig_trigger": {{"use_efficiency": true, "destroy_speeds": [{{"block": "minecraft:stone", "speed": 2, "zz_inner": 1}}], "yy_outer": 2}}}}}}}}"#
        );
        // The hand-written side-car is a deprecated name of the 1.20.30–1.26.0 roots: the
        // item is refused; the reading of what is inside is still told.
        let diagnostics = findings(&text, &cx);
        assert!(
            diagnostics.iter().any(|f| f.severity == Severity::Error
                && f.code.id() == "item/deprecated_components_deprecated"),
            "{version}: {diagnostics:#?}"
        );
        let told: Vec<String> = diagnostics
            .iter()
            .filter(|f| f.code.id() == "core/unrecognized_field")
            .map(|f| format!("{:?} at {}", f.severity, f.location.json_path))
            .collect();
        let at = "minecraft:item/__deprecated_components/legacy_on_dig_trigger";
        // (A file below 1.20.60 reads an unknown member without a word, here as anywhere.)
        let expected = if version == "1.20.30" {
            Vec::new()
        } else {
            vec![
                // The side-car's notes are informations of ours.
                format!("Info at {at}/destroy_speeds/0/zz_inner"),
                format!("Info at {at}/yy_outer"),
            ]
        };
        assert_eq!(told, expected, "{version}");
    }
}

/// A `minecraft:rarity` that is no rarity: the value, what is told at the key, and the member
/// as it is written. The game's legacy reader takes the key of such a file in silence; the
/// 1.21.30 reader's word on it is the crate's own, an information.
const EXPECTED_UNREAD: &[(&str, &[&str], &str)] = &[
    (
        "[1]",
        &["core/syntax Info at minecraft:item/components/minecraft:rarity"],
        "absent",
    ),
    (
        "true",
        &["core/invalid_type Info at minecraft:item/components/minecraft:rarity"],
        "absent",
    ),
    (
        r#"{"value": {"a": 1}}"#,
        &[
            "core/dropped_component Info at minecraft:item/components/minecraft:rarity",
            "core/invalid_type Info at minecraft:item/components/minecraft:rarity/value/a",
        ],
        "absent",
    ),
];

/// The legacy item format: the whole document is the legacy reading (`format`,
/// `format_item`), and both writers write it whole: the text reads back to the same value
/// and is written as itself.
#[test]
fn a_legacy_format_item_is_written_whole_by_both_writers() {
    let cx = allowlisted();
    let path = PackPath::new("items/x.json").unwrap();
    let text = r#"{"format_version": "1.10", "minecraft:item": {"description": {"identifier": "demo:x", "category": "Nature"}, "components": {"minecraft:max_stack_size": 16, "minecraft:foil": true, "minecraft:block": "minecraft:stone", "minecraft:use_duration": 32, "minecraft:hand_equipped": true, "zz_unknown": {"a": 1}}}}"#;
    let first = loads(text, &cx);
    assert_eq!(
        first.value().legacy.format,
        bedrock_addon_family_item::ItemFormat::Legacy
    );
    assert!(first.value().legacy.format_item.is_some());
    let written = first.to_json().expect("written");
    for token in [
        "Nature",
        "minecraft:foil",
        "minecraft:stone",
        "32",
        "hand_equipped",
        "zz_unknown",
    ] {
        assert!(written.contains(token), "{token}: {written}");
    }
    let second = loads(&written, &cx);
    assert_eq!(second.value(), first.value(), "{written}");
    assert_eq!(second.to_json().expect("written again"), written);
    // As it was read.
    let as_read = Item::parse_as_written(&path, text.as_bytes(), &cx).expect("loads");
    let written = as_read.to_json().expect("written as read");
    let again = Item::parse_as_written(&path, written.as_bytes(), &cx).expect("reads");
    assert_eq!(again.value(), as_read.value(), "{written}");
    assert_eq!(again.to_json().expect("written again"), written);
}

/// What the newest format says of a member it does not know is its own judgement: a file
/// below 1.20.60 is read without a word about an unknown member (its format accepts it), and
/// the member is kept. The newest format is a strict slice that refuses it, so the
/// newest writer refuses the item there; the as-written writer keeps it. Nothing is
/// lost in silence.
#[test]
fn an_unknown_description_member_of_an_older_file_is_kept_and_told_only_by_the_newest_format() {
    let cx = third_party();
    let text = r#"{"format_version": "1.20.30", "minecraft:item": {"description": {"identifier": "demo:x", "zz_unknown": 1}, "components": {"minecraft:max_stack_size": 16}}}"#;
    let first = loads(text, &cx);
    // (the load's one word is the note that the newest format refuses it; an
    // information of ours since)
    let told: Vec<String> = first
        .diagnostics()
        .iter()
        .filter(|f| f.code.id() == "core/invalid_value")
        .map(|f| format!("{} {:?} at {}", f.code.id(), f.phase, f.location.json_path))
        .collect();
    assert_eq!(
        told,
        ["core/invalid_value Validate at minecraft:item/description/zz_unknown"]
    );
    let written = written_at_1_26_0(&first.to_json());
    assert!(written.contains(r#""zz_unknown": 1"#), "{written}");
    let as_read = written_as_read(text, &cx);
    assert!(as_read.contains(r#""zz_unknown": 1"#), "{as_read}");
}

/// The same for a `__deprecated_components` a file writes itself: the file's format binds it
/// as a deprecated name, which refuses the item; the newest format binds no such member.
#[test]
fn a_hand_written_side_car_refuses_the_item_as_a_deprecated_name() {
    let cx = allowlisted();
    let text = r#"{"format_version": "1.21.10", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:max_stack_size": 16}, "__deprecated_components": {"render_offsets": "tools"}}}"#;
    // The `deprecateName` of the root is cereal's 0x200, logged for a JSON
    // object and an error in both masks: the item is refused,
    // whatever the member holds.
    let d = rejects(text, &cx);
    assert!(
        d.iter().any(|f| f.severity == Severity::Error
            && f.code.id() == "item/deprecated_components_deprecated"
            && f.location.json_path.to_string() == "minecraft:item/__deprecated_components"),
        "{d:#?}"
    );
}
