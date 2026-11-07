//! The fallbacks of the legacy reader and the upgrade steps: a member
//! a step cannot convert as its folder describes (a list that is no list, an element that is
//! no string or no number) is read as the step reads it, and the step says so at the
//! member, as the file has it. Every test loads a legacy document of the version the step
//! upgrades from through `Block::parse`.

use bedrock_addon::diagnostic::{Phase, Severity};
use bedrock_addon::document::Content;
use bedrock_addon::json::Value;

use super::common::{allow_listed, document, loads};

const BLOCK: &str = "minecraft:block";

/// The upgrade-phase `InvalidType` informations (ours) of the block `payload` of
/// `version`, by path, and
/// the upgraded block in the newest format's shape.
#[track_caller]
fn misread(version: &str, payload: &str) -> (Vec<String>, Value) {
    let block = loads(&document(version, payload), &allow_listed());
    let told = block
        .diagnostics()
        .iter()
        .filter(|f| {
            f.phase == Phase::Upgrade
                && f.severity == Severity::Info
                // By legacy name: the variant and the definitions that were it.
                && f.code.name() == "InvalidType"
        })
        .map(|f| f.location.json_path.to_string())
        .collect();
    (told, super::common::newest_shape(&block))
}

/// A JSON value from text.
fn json(text: &str) -> Value {
    facet_json::from_str(text).expect("JSON")
}

/// The value at a `/`-separated path of the written payload.
fn at<'v>(written: &'v Value, path: &str) -> Option<&'v Value> {
    let mut value = written.as_object()?.get(BLOCK)?;
    for step in path.split('/') {
        value = match step.parse::<usize>() {
            Ok(index) if value.is_array() => value.as_array()?.get(index)?,
            _ => value.as_object()?.get(step)?,
        };
    }
    Some(value)
}

/// A payload with `components` and `more` members beside the description.
fn payload(components: &str, more: &str) -> String {
    format!(r#"{{"description": {{"identifier": "demo:x"}}, "components": {components}{more}}}"#)
}

#[test]
fn a_v1_names_list_that_is_no_list_of_strings_is_read_as_before_and_told() {
    const LIST: &str = "ordered_v1_custom_components";
    // No array: the empty list, told at the member.
    for value in [r#""a:b""#, "7", "null", "{}"] {
        let (told, written) = misread(
            "1.21.90",
            &payload("{}", &format!(r#", "{LIST}": {value}"#)),
        );
        assert_eq!(told, [format!("{BLOCK}/{LIST}")], "{value}");
        assert_eq!(at(&written, LIST), None, "{value}");
    }
    // An element that is no string is left out, told at the element; the others are kept.
    let (told, written) = misread(
        "1.21.90",
        &payload("{}", &format!(r#", "{LIST}": ["a:b", 5, null, "c:d"]"#)),
    );
    assert_eq!(
        told,
        [format!("{BLOCK}/{LIST}/1"), format!("{BLOCK}/{LIST}/2")]
    );
    assert_eq!(at(&written, LIST), Some(&json(r#"["a:b", "c:d"]"#)));
    // The same in a permutation, at the permutation's member.
    let (told, written) = misread(
        "1.21.90",
        &payload(
            "{}",
            &format!(
                r#", "permutations": [{{"condition": "1", "components": {{}}, "{LIST}": [true, "a:b"]}}]"#
            ),
        ),
    );
    assert_eq!(told, [format!("{BLOCK}/permutations/0/{LIST}/0")]);
    assert_eq!(
        at(&written, &format!("permutations/0/{LIST}")),
        Some(&json(r#"["a:b"]"#))
    );
    // A list of strings is read without a word.
    let (told, written) = misread(
        "1.21.90",
        &payload("{}", &format!(r#", "{LIST}": ["a:b"]"#)),
    );
    assert!(told.is_empty(), "{told:?}");
    assert_eq!(at(&written, LIST), Some(&json(r#"["a:b"]"#)));
}

#[test]
fn a_v1_custom_components_list_that_is_no_list_of_strings_is_removed_and_told() {
    const LIST: &str = "minecraft:custom_components";
    // No array: the component is removed and no custom component is written.
    for value in [r#""my:comp""#, "{}", "7", "null"] {
        let (told, written) = misread(
            "1.21.60",
            &payload(&format!(r#"{{"{LIST}": {value}}}"#), ""),
        );
        assert_eq!(told, [format!("{BLOCK}/components/{LIST}")], "{value}");
        assert_eq!(at(&written, &format!("components/{LIST}")), None, "{value}");
        assert_eq!(at(&written, "components/my:comp"), None, "{value}");
        assert_eq!(
            at(&written, "ordered_v1_custom_components"),
            None,
            "{value}"
        );
    }
    // An element that is no string names no component: left out, told at the element.
    let (told, written) = misread(
        "1.21.60",
        &payload(
            &format!(r#"{{"{LIST}": ["my:comp", 7, null, "my:other"]}}"#),
            "",
        ),
    );
    assert_eq!(
        told,
        [
            format!("{BLOCK}/components/{LIST}/1"),
            format!("{BLOCK}/components/{LIST}/2")
        ]
    );
    assert_eq!(at(&written, "components/my:comp"), Some(&json("{}")));
    assert_eq!(at(&written, "components/my:other"), Some(&json("{}")));
    assert_eq!(
        at(&written, "ordered_v1_custom_components"),
        Some(&json(r#"["my:comp", "my:other"]"#))
    );
    // The same in a permutation.
    let (told, written) = misread(
        "1.21.60",
        &payload(
            "{}",
            &format!(
                r#", "permutations": [{{"condition": "1", "components": {{"{LIST}": [false, "my:comp"]}}}}]"#
            ),
        ),
    );
    assert_eq!(
        told,
        [format!("{BLOCK}/permutations/0/components/{LIST}/0")]
    );
    assert_eq!(
        at(&written, "permutations/0/ordered_v1_custom_components"),
        Some(&json(r#"["my:comp"]"#))
    );
    // A list of strings converts without a word.
    let (told, written) = misread(
        "1.21.60",
        &payload(&format!(r#"{{"{LIST}": ["my:comp"]}}"#), ""),
    );
    assert!(told.is_empty(), "{told:?}");
    assert_eq!(at(&written, "components/my:comp"), Some(&json("{}")));
}

#[test]
fn a_tags_component_that_is_no_list_beside_a_tag_component_is_replaced_and_told() {
    const TAGS: &str = "minecraft:tags";
    for value in [r#""stone""#, "5", "null", "{}"] {
        let (told, written) = misread(
            "1.26.0",
            &payload(
                &format!(r#"{{"tag:ore": {{}}, "{TAGS}": {value}, "tag:metal": {{}}}}"#),
                "",
            ),
        );
        assert_eq!(told, [format!("{BLOCK}/components/{TAGS}")], "{value}");
        assert_eq!(
            at(&written, &format!("components/{TAGS}")),
            Some(&json(r#"["ore", "metal"]"#)),
            "{value}"
        );
    }
    // In a permutation, at the permutation's component.
    let (told, written) = misread(
        "1.26.0",
        &payload(
            "{}",
            &format!(
                r#", "permutations": [{{"condition": "1", "components": {{"{TAGS}": 5, "tag:ore": {{}}}}}}]"#
            ),
        ),
    );
    assert_eq!(told, [format!("{BLOCK}/permutations/0/components/{TAGS}")]);
    assert_eq!(
        at(&written, &format!("permutations/0/components/{TAGS}")),
        Some(&json(r#"["ore"]"#))
    );
    // A list takes the tag components after its own names, without a word.
    let (told, written) = misread(
        "1.26.0",
        &payload(&format!(r#"{{"tag:ore": {{}}, "{TAGS}": ["stone"]}}"#), ""),
    );
    assert!(told.is_empty(), "{told:?}");
    assert_eq!(
        at(&written, &format!("components/{TAGS}")),
        Some(&json(r#"["stone", "ore"]"#))
    );
}

#[test]
fn a_description_properties_that_is_no_object_is_discarded_and_told() {
    let described = |properties: &str| {
        format!(
            r#"{{"description": {{"identifier": "demo:x", "properties": {properties}}}, "components": {{}}}}"#
        )
    };
    for value in ["7", r#""demo:lit""#, "[false, true]", "null"] {
        let (told, written) = misread("1.20.10", &described(value));
        assert_eq!(told, [format!("{BLOCK}/description/properties")], "{value}");
        assert_eq!(at(&written, "description/properties"), None, "{value}");
        assert_eq!(at(&written, "description/states"), None, "{value}");
    }
    // An object becomes the states, without a word.
    let (told, written) = misread("1.20.10", &described(r#"{"demo:lit": [false, true]}"#));
    assert!(told.is_empty(), "{told:?}");
    assert_eq!(
        at(&written, "description/states"),
        Some(&json(r#"{"demo:lit": [false, true]}"#))
    );
}

/// A 1.18.0 block with a `minecraft:ticking` whose `range` is `range`.
fn ticking(range: &str) -> String {
    format!(
        r#"{{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:ticking": {{"on_tick": {{"event": "e"}}, "range": {range}, "looping": true}}}}, "events": {{"e": {{}}}}}}"#
    )
}

#[test]
fn a_ticking_range_element_that_is_no_number_is_left_out_and_told() {
    let range = format!("{BLOCK}/components/minecraft:ticking/range");
    // The one number left is the first end: 2 s are 40 ticks.
    for (value, index) in [(r#"["x", 2]"#, 0), ("[2, null]", 1), ("[2, [1]]", 1)] {
        let (told, written) = misread("1.18.0", &ticking(value));
        assert_eq!(told, [format!("{range}/{index}")], "{value}");
        assert_eq!(
            at(&written, "deprecated/queued_ticking/min_ticks"),
            Some(&json("40")),
            "{value}"
        );
    }
    let (told, _) = misread("1.18.0", &ticking(r#"[true, "x"]"#));
    assert_eq!(told, [format!("{range}/0"), format!("{range}/1")]);
    // Two numbers convert without a word.
    let (told, written) = misread("1.18.0", &ticking("[1, 2]"));
    assert!(told.is_empty(), "{told:?}");
    assert_eq!(
        at(&written, "deprecated/queued_ticking/max_ticks"),
        Some(&json("40"))
    );
}

/// A 1.26.0 block with a `minecraft:queued_ticking` with the members `members` beside
/// `on_tick`.
fn queued_ticking(members: &str) -> String {
    format!(
        r#"{{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:queued_ticking": {{"on_tick": {{"event": "e"}}{members}}}}}, "events": {{"e": {{}}}}}}"#
    )
}

#[test]
fn a_queued_ticking_interval_range_that_is_no_array_gives_no_ticks_and_is_told() {
    let member = format!("{BLOCK}/components/minecraft:queued_ticking/interval_range");
    let ticks = |written: &Value| {
        (
            at(written, "deprecated/queued_ticking/min_ticks").cloned(),
            at(written, "deprecated/queued_ticking/max_ticks").cloned(),
        )
    };
    // What a queued ticking without the member holds: the reference for "no ticks".
    let (told, written) = misread("1.26.0", &queued_ticking(""));
    assert!(told.is_empty(), "{told:?}");
    let none = ticks(&written);
    for value in ["7", r#""1-2""#, r#"{"min": 1}"#, "null"] {
        let (told, written) = misread(
            "1.26.0",
            &queued_ticking(&format!(r#", "interval_range": {value}"#)),
        );
        assert_eq!(told, std::slice::from_ref(&member), "{value}");
        assert_eq!(ticks(&written), none, "{value}");
    }
    // An array gives its two ends, without a word.
    let (told, written) = misread("1.26.0", &queued_ticking(r#", "interval_range": [3, 9]"#));
    assert!(told.is_empty(), "{told:?}");
    assert_eq!(ticks(&written), (Some(json("3")), Some(json("9"))));
}

/// More values a step reads as another value, or takes with a conversion, each told
/// at the member the file has; the block holds what the step made of it.
#[test]
fn a_value_a_step_reads_as_another_value_is_told() {
    // (the payload's members after the description, the path told, the code, the severity)
    // (A `flammable` odds member written `null` is there for the game's check; see
    // `upgrade.rs`.)
    let rows: [(&str, &str, &str, Severity); 6] = [
        (
            r#""components": {"minecraft:block_light_absorption": true}"#,
            "minecraft:block/components/minecraft:block_light_absorption",
            "InvalidType",
            Severity::Info,
        ),
        (
            r#""components": {"minecraft:block_light_absorption": 3.7}"#,
            "minecraft:block/components/minecraft:block_light_absorption",
            "InvalidValue",
            Severity::Info,
        ),
        (
            r#""components": {"minecraft:block_light_emission": {"emission": 0.5, "a": 1}}"#,
            // "Members of component %s will be thrown away as part of the upgrade": the
            // game's level-3 line, once at the component.
            "minecraft:block/components/minecraft:block_light_emission",
            "DroppedField",
            Severity::Error,
        ),
        (
            r#""components": {}, "events": 7"#,
            "minecraft:block/events",
            "InvalidType",
            Severity::Info,
        ),
        (
            r#""components": {"minecraft:rotation": null}"#,
            "minecraft:block/components/minecraft:rotation",
            "InvalidType",
            Severity::Info,
        ),
        (
            r#""components": {"tag:ore": 7}"#,
            "minecraft:block/components/tag:ore",
            "InvalidType",
            Severity::Info,
        ),
    ];
    for (members, at, code, severity) in rows {
        let text = format!(
            r#"{{"format_version": "1.16.100", "minecraft:block": {{"description": {{"identifier": "demo:x"}}, {members}}}}}"#
        );
        let block = bedrock_addon::families::block::Block::parse(
            &bedrock_addon::storage::PackPath::from_static("blocks/x.json"),
            text.as_bytes(),
            &allow_listed(),
        )
        .unwrap_or_else(|rejected| panic!("{members}: {:#?}", rejected.diagnostics));
        let told: Vec<String> = block
            .diagnostics()
            .iter()
            .filter(|f| f.phase == Phase::Upgrade && f.location.json_path.to_string() == at)
            .filter(|f| f.severity == severity)
            .map(|f| f.code.name().to_owned())
            .collect();
        assert!(
            told.contains(&code.to_owned()),
            "{members}: {told:?}; {:#?}",
            block.diagnostics()
        );
    }
    // A beta flag that is no bool is no flag: the member is unknown, told as such
    // (no misread, nothing reads it).
    let text = r#"{"format_version": "1.16.100", "minecraft:block": {"description": {"identifier": "demo:x", "use_beta_features": "true"}, "components": {}}}"#;
    let block = bedrock_addon::families::block::Block::parse(
        &bedrock_addon::storage::PackPath::from_static("blocks/x.json"),
        text.as_bytes(),
        &allow_listed(),
    )
    .unwrap_or_else(|rejected| panic!("{:#?}", rejected.diagnostics));
    assert!(block.diagnostics().iter().any(|f| {
        f.code.name() == "UnrecognizedField"
            && f.location.json_path.to_string() == "minecraft:block/description/use_beta_features"
    }));
    assert!(
        !block
            .diagnostics()
            .iter()
            .any(|f| f.code.name() == "InvalidType")
    );
}

/// The category of a legacy `minecraft:creative_category`
/// component that the 1.19.30 step refuses is told at the member the FILE has (the component
/// of the base or of the permutation that wrote it), not at `description/menu_category`,
/// which the file does not have. A file that has `description.menu_category` itself is told
/// there, as before.
#[test]
fn a_legacy_creative_category_is_told_at_the_member_the_file_has() {
    let parse = |text: &str| {
        bedrock_addon::families::block::Block::parse(
            &bedrock_addon::storage::PackPath::from_static("blocks/x.json"),
            text.as_bytes(),
            &allow_listed(),
        )
        .unwrap_or_else(|rejected| panic!("{:#?}", rejected.diagnostics))
    };
    let told = |text: &str| -> Vec<String> {
        parse(text)
            .diagnostics()
            .iter()
            .filter(|f| f.code.name() == "InvalidValue" && f.phase == Phase::Upgrade)
            .map(|f| f.location.json_path.to_string())
            .collect()
    };
    assert_eq!(
        told(
            r#"{"format_version": "1.16.100", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:creative_category": {"category": "bogus"}}}}"#
        ),
        ["minecraft:block/components/minecraft:creative_category/category"]
    );
    // The last permutation that carries one wins: told there.
    assert_eq!(
        told(
            r#"{"format_version": "1.16.100", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:creative_category": {"category": "nature"}}, "permutations": [{"condition": "1", "components": {}}, {"condition": "1", "components": {"minecraft:creative_category": {"category": ""}}}]}}"#
        ),
        ["minecraft:block/permutations/1/components/minecraft:creative_category/category"]
    );
    assert_eq!(
        told(
            r#"{"format_version": "1.19.20", "minecraft:block": {"description": {"identifier": "demo:x", "menu_category": {"category": "bogus"}}, "components": {}}}"#
        ),
        ["minecraft:block/description/menu_category/category"]
    );
    // What only a file of 1.26.20 is held to (a category is required, a group is a
    // namespaced name) is told there too, and the block is not written.
    let upgrade = |text: &str| -> Vec<String> {
        parse(text)
            .diagnostics()
            .iter()
            .filter(|f| f.phase == Phase::Upgrade && f.severity == Severity::Info)
            .map(|f| format!("{} {}", f.code.name(), f.location.json_path))
            .collect()
    };
    assert_eq!(
        upgrade(
            r#"{"format_version": "1.16.100", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:creative_category": {"group": "g"}}}}"#
        ),
        [
            // The group the 1.21.30 step writes under vanilla's namespace.
            "DeprecatedMember minecraft:block/components/minecraft:creative_category/group",
            "MissingRequiredField minecraft:block/components/minecraft:creative_category/category"
        ]
    );
    assert_eq!(
        upgrade(
            r#"{"format_version": "1.19.20", "minecraft:block": {"description": {"identifier": "demo:x", "menu_category": {"group": "g"}}, "components": {}}}"#
        ),
        [
            "DeprecatedMember minecraft:block/description/menu_category/group",
            "MissingRequiredField minecraft:block/description/menu_category/category"
        ]
    );
    let refused = parse(
        r#"{"format_version": "1.16.100", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:creative_category": {"group": "g"}}}}"#,
    );
    assert!(refused.to_json().is_err(), "told and not written");
    // Nothing of the step's bookkeeping is in the block.
    let block = parse(
        r#"{"format_version": "1.16.100", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:creative_category": {"category": "nature"}}}}"#,
    );
    let written = block.to_json().unwrap();
    assert!(!written.contains("__"), "{written}");
    assert!(
        block
            .diagnostics()
            .iter()
            .all(|f| f.code.name() != "UnrecognizedField")
    );
}

/// The `InvalidType` finding of a component of a 1.26.20
/// block whose value is of another type has the span of the value's token, not a span of no
/// length.
#[test]
fn an_invalid_type_of_a_cereal_component_has_the_span_of_its_token() {
    for (component, token) in [
        (r#""minecraft:friction": 1e39"#, "1e39"),
        (r#""minecraft:light_emission": "x""#, r#""x""#),
        (r#""minecraft:friction": "x""#, r#""x""#),
    ] {
        let text = document(
            "1.26.20",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{{component}}}}}"#
            ),
        );
        let found = match super::common::load(&text, &super::common::context()) {
            Ok(document) => document.diagnostics().clone(),
            Err(rejected) => rejected.diagnostics,
        };
        let spans: Vec<(String, &str)> = found
            .iter()
            .filter(|f| {
                f.location
                    .json_path
                    .to_string()
                    .contains("components/minecraft:")
            })
            .map(|f| {
                let span = f.location.span.expect("a span");
                (
                    f.code.name().to_owned(),
                    &text[span.offset as usize..(span.offset + span.len) as usize],
                )
            })
            .collect();
        assert!(!spans.is_empty(), "{component}: {found:#?}");
        for (code, under) in &spans {
            assert_eq!(under, &token, "{component}: {code} is on `{under}`");
        }
    }
}
