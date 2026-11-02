//! Extra pins: each test here fails for one
//! change of `src/poi/` that every other test let through.

use bedrock_addon::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon::families::poi::{PoiTag, PoiType};
use bedrock_addon::json::JsonSlice;
use bedrock_addon::version::{FormatVersion, FormatVersionInput};
use bedrock_addon::versioning::{ParseCx, VersionStruct};

use super::common::{block, block_of, context, is_closing, names, rejected, tag_of};

/// The errors of `findings` as `(code, JSON path)`.
fn errors_at(findings: &Diagnostics) -> Vec<(&'static str, String)> {
    findings
        .iter()
        .filter(|d| d.severity == Severity::Error && !is_closing(d))
        .map(|d| (d.code.name(), d.location.json_path.to_string()))
        .collect()
}

/// Parses `payload` with the struct's own parse, below `key`.
fn parse_struct<T: VersionStruct>(key: &str, payload: &str) -> (Option<T>, Diagnostics) {
    let load = context();
    let mut at = JsonPath::new();
    at.push(key);
    let cx = ParseCx {
        load: &load,
        written: FormatVersionInput {
            version: Some(FormatVersion::new(1, 26, 40)),
            beta_literal: false,
            prerelease: false,
        },
        original_version: FormatVersion::new(1, 26, 40),
        at,
    };
    let mut d = Diagnostics::default();
    let value = T::parse(JsonSlice::new(payload), &cx, &mut d);
    (value, d)
}

/// The length of the long cycle of tags.
const CYCLE_LENGTH: usize = 600;

/// The files of a long cycle: t0 ⊂ t1 ⊂ … ⊂ t599 ⊂ t0, a type on every tag, and a tail below t0.
fn long_cycle_files() -> Vec<(String, String)> {
    let mut files: Vec<(String, String)> = Vec::new();
    for i in 0..CYCLE_LENGTH {
        files.push((
            format!("poi/tags/t{i:04}.json"),
            tag_of(
                &format!("test:t{i}"),
                &format!(r#", "parents": ["test:t{}"]"#, (i + 1) % CYCLE_LENGTH),
            ),
        ));
        files.push((
            format!("poi/blocks/i{i:04}.json"),
            block_of(
                &format!("test:i{i}"),
                &format!(r#""tags": ["test:t{i}"],"#),
                &format!(r#"["test:block{i}"]"#),
            ),
        ));
    }
    files.push((
        "poi/tags/tail.json".to_owned(),
        tag_of("test:tail", r#", "parents": ["test:t0"]"#),
    ));
    files.push((
        "poi/blocks/tail.json".to_owned(),
        block_of(
            "test:tailed",
            r#""tags": ["test:tail"],"#,
            r#"["test:tail_block"]"#,
        ),
    ));
    files
}

/// The termination conditions of the tag expansion and of the cycle search have their direct
/// test here. A tag naming
/// itself, a two-tag cycle and a 600-tag cycle (with a type on every tag and a tail hanging
/// off it) each load quickly with the documented finding, and no tag inherits round the cycle.
#[test]
fn cyclic_tag_graphs_terminate_quickly_with_their_findings() {
    use bedrock_addon::id::{PoiTagId, PoiTypeId};
    use std::time::{Duration, Instant};

    let started = Instant::now();
    // A tag naming itself, as parent and as child.
    let stack = super::common::stack(vec![super::common::pack(
        10,
        &[(
            "poi/tags/a.json",
            &tag_of(
                "test:a",
                r#", "parents": ["test:a"], "children": ["test:a"]"#,
            ),
        )],
    )]);
    let poi = stack.poi();
    assert_eq!(
        names(poi.diagnostics(), Severity::Info),
        ["ReferenceCycle", "ReferenceCycle"]
    );
    let a = PoiTagId::from_static("test:a");
    assert!(poi.parents(&a).is_empty() && poi.children(&a).is_empty());

    // Two tags naming each other, each side saying it twice over.
    let stack = super::common::stack(vec![super::common::pack(
        10,
        &[
            (
                "poi/tags/a.json",
                &tag_of(
                    "test:a",
                    r#", "parents": ["test:b"], "children": ["test:b"]"#,
                ),
            ),
            (
                "poi/tags/b.json",
                &tag_of(
                    "test:b",
                    r#", "parents": ["test:a"], "children": ["test:a"]"#,
                ),
            ),
        ],
    )]);
    let poi = stack.poi();
    let texts: Vec<String> = poi.diagnostics().map(|d| d.code.to_string()).collect();
    assert_eq!(
        texts,
        [
            "test:a was a part of a cycle",
            "test:b was a part of a cycle"
        ]
    );

    let files = long_cycle_files();
    let borrowed: Vec<(&str, &str)> = files
        .iter()
        .map(|(path, text)| (path.as_str(), text.as_str()))
        .collect();
    let stack = super::common::stack(vec![super::common::pack(10, &borrowed)]);
    let poi = stack.poi();
    assert_eq!(
        names(poi.diagnostics(), Severity::Info).len(),
        CYCLE_LENGTH,
        "one information per tag of the cycle, none for the tail"
    );
    assert!(
        poi.diagnostics()
            .all(|d| d.code.name() == "ReferenceCycle" && d.severity == Severity::Info)
    );
    for i in [1, 299, 599] {
        assert_eq!(
            poi.members(&PoiTagId::parse(format!("test:t{i}"))),
            [PoiTypeId::parse(format!("test:i{i}"))],
            "only its own type"
        );
    }
    assert_eq!(
        poi.members(&PoiTagId::from_static("test:t0")),
        [
            PoiTypeId::from_static("test:i0"),
            PoiTypeId::from_static("test:tailed")
        ],
        "its own type and the tail's, nothing round the cycle"
    );
    // A guard against a hang, not a cost bound: 450 ms measured in the dev profile,
    // 44 times under the bound.
    assert!(
        started.elapsed() < Duration::from_secs(20),
        "{:?}",
        started.elapsed()
    );
}

/// Mutant "names: only the last entry decides": a non-string entry anywhere in a list keeps
/// the typed parse away, so it is reported once, whatever follows it.
#[test]
fn a_non_string_entry_is_one_type_error_whatever_follows_it() {
    let d = rejected::<PoiType>(&block_of("test:forge", "", r#"[5, "minecraft:anvil"]"#));
    assert_eq!(
        errors_at(&d),
        [("InvalidType", "minecraft:poi_block/blocks/0".to_owned())]
    );
    let d = rejected::<PoiType>(&block_of(
        "test:forge",
        r#""tags": [{}, "test:civic"],"#,
        r#"["minecraft:anvil"]"#,
    ));
    assert_eq!(
        errors_at(&d),
        [("InvalidType", "minecraft:poi_block/tags/0".to_owned())]
    );
    for member in ["children", "parents"] {
        let d = rejected::<PoiTag>(&tag_of(
            "test:civic",
            &format!(r#", "{member}": [[], "test:a", true, "test:b"]"#),
        ));
        assert_eq!(
            errors_at(&d),
            [
                ("InvalidType", format!("minecraft:poi_tag/{member}/0")),
                ("InvalidType", format!("minecraft:poi_tag/{member}/2")),
            ]
        );
    }
}

/// Mutant "strict: value despite an error": the struct's own parse gives no value once it has
/// reported an error (a pattern miss or an unknown member leaves the typed parse able to
/// build one).
#[test]
fn the_struct_parse_gives_no_value_once_it_reported_an_error() {
    let (value, d) = parse_struct::<PoiType>(
        "minecraft:poi_block",
        r#"{"description": {"identifier": "forge"}, "blocks": ["minecraft:anvil"]}"#,
    );
    assert!(value.is_none(), "a pattern miss");
    assert_eq!(names(d.iter(), Severity::Error), ["InvalidValue"]);
    let (value, d) = parse_struct::<PoiType>(
        "minecraft:poi_block",
        r#"{"description": {"identifier": "test:forge"}, "blocks": ["minecraft:anvil"], "radius": 2}"#,
    );
    // An unknown member is the struct's warning; the slice's mask (0xfffc) makes it the
    // loader's error.
    assert!(value.is_some(), "an unknown member alone");
    assert_eq!(names(d.iter(), Severity::Warning), ["UnrecognizedField"]);
    let d = rejected::<PoiType>(&block(
        r#"{"description": {"identifier": "test:forge"}, "blocks": ["minecraft:anvil"], "radius": 2}"#,
    ));
    assert_eq!(names(d.iter(), Severity::Error), ["UnrecognizedField"]);
    let (value, d) = parse_struct::<PoiTag>(
        "minecraft:poi_tag",
        r#"{"description": {"identifier": "test:civic"}, "parents": ["Village"]}"#,
    );
    assert!(value.is_none(), "a pattern miss in a list");
    assert_eq!(names(d.iter(), Severity::Error), ["InvalidValue"]);
    // A clean payload is a value and no finding.
    let (value, d) = parse_struct::<PoiTag>(
        "minecraft:poi_tag",
        r#"{"description": {"identifier": "test:civic"}, "parents": ["test:town"]}"#,
    );
    assert!(value.is_some() && d.is_empty());
}

/// Mutant "X5: index lost": the claim is reported at the entry that made it, not at the list's
/// first.
#[test]
fn a_claimed_block_is_reported_at_its_own_entry() {
    let stack = super::common::stack(vec![super::common::pack(
        10,
        &[
            (
                "poi/blocks/a.json",
                &block_of("test:a", "", r#"["minecraft:anvil"]"#),
            ),
            (
                "poi/blocks/b.json",
                &block_of(
                    "test:b",
                    "",
                    r#"["minecraft:bell", "minecraft:lectern", "minecraft:anvil"]"#,
                ),
            ),
        ],
    )]);
    let claimed: Vec<String> = stack
        .poi()
        .diagnostics()
        .filter(|d| d.code.name() == "PoiBlockClaimedTwice")
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(claimed, ["minecraft:poi_block/blocks/2"]);
}

/// Mutants "E7: as-written gate off" and "E7: upgrade gate off": the two halves of the load
/// are gated like the whole.
#[test]
fn the_as_written_parse_and_the_upgrade_are_gated_too() {
    use bedrock_addon::families::poi::history::{PoiTagAny, PoiTypeAny};
    use bedrock_addon::load::LoadOptions;
    use bedrock_addon::storage::PackPath;

    let on = context();
    let off = super::common::context_with(&LoadOptions::default(), false);
    let path = PackPath::new("poi/document.json").unwrap();
    let forge = block_of("test:forge", "", r#"["minecraft:anvil"]"#);
    let civic = tag_of("test:civic", "");

    let closed = PoiType::parse_as_written(&path, forge.as_bytes(), &off).expect_err("gated");
    assert_eq!(
        names(closed.diagnostics.iter(), Severity::Info),
        ["MissingToggles"]
    );
    assert_eq!(closed.diagnostics.len(), 1);
    let closed = PoiTag::parse_as_written(&path, civic.as_bytes(), &off).expect_err("gated");
    assert_eq!(
        names(closed.diagnostics.iter(), Severity::Info),
        ["MissingToggles"]
    );

    let written = PoiType::parse_as_written(&path, forge.as_bytes(), &on).expect("parses");
    let closed = PoiTypeAny::upgrade(written, &off).expect_err("gated");
    assert_eq!(
        names(closed.diagnostics.iter(), Severity::Info),
        ["MissingToggles"]
    );
    assert_eq!(closed.diagnostics.len(), 1);
    let written = PoiTag::parse_as_written(&path, civic.as_bytes(), &on).expect("parses");
    let closed = PoiTagAny::upgrade(written, &off).expect_err("gated");
    assert_eq!(
        names(closed.diagnostics.iter(), Severity::Info),
        ["MissingToggles"]
    );
}

/// Mutant "text: no depth check": well-formed JSON nested past the limit is refused before
/// anything reads it (the same file one level inside the limit gets the member's own error).
#[test]
fn json_nested_past_the_limit_is_malformed_input() {
    let nested = |depth: usize| {
        block_of(
            "test:forge",
            "",
            &format!("{}{}", "[".repeat(depth), "]".repeat(depth)),
        )
    };
    let d = rejected::<PoiType>(&nested(200));
    assert_eq!(names(d.iter(), Severity::Error), ["MalformedInput"]);
    let d = rejected::<PoiType>(&nested(20));
    assert_eq!(
        errors_at(&d),
        [("InvalidType", "minecraft:poi_block/blocks/0".to_owned())]
    );
}

/// Mutant "X6: merged document's findings lost": the findings of a tag document that is
/// unioned into an earlier one of its pack are kept.
#[test]
fn a_merged_tag_document_keeps_its_findings() {
    let stack = super::common::stack(vec![super::common::pack(
        10,
        &[
            ("poi/tags/t_a.json", &tag_of("test:t", "")),
            (
                "poi/tags/t_b.json",
                &tag_of("test:t", r#", "parents": ["test:t"]"#),
            ),
        ],
    )]);
    let poi = stack.poi();
    let texts: Vec<String> = poi.diagnostics().map(|d| d.code.to_string()).collect();
    assert_eq!(texts, ["test:t was a part of a cycle"]);
    assert_eq!(poi.tags.len(), 1);
}

/// Mutant "locate: no pack": a document's own findings carry its pack and its file, for a
/// loaded document and for a rejected one.
#[test]
fn a_documents_findings_carry_its_pack_and_file() {
    let low = super::common::pack(
        20,
        &[("poi/blocks/empty.json", &block_of("test:empty", "", "[]"))],
    );
    let top = super::common::pack(
        10,
        &[(
            "poi/blocks/bad.json",
            &block_of("forge", "", r#"["minecraft:anvil"]"#),
        )],
    );
    let stack = super::common::stack(vec![top, low]);
    let mut found: Vec<(&str, Option<usize>, Option<&str>)> = stack
        .poi()
        .diagnostics()
        .map(|d| {
            (
                d.code.name(),
                d.location
                    .pack
                    .as_ref()
                    .and_then(bedrock_addon::pack::PackRef::index),
                d.location
                    .path
                    .as_ref()
                    .map(bedrock_addon::storage::PackPath::as_str),
            )
        })
        .collect();
    found.sort();
    assert_eq!(
        found,
        [
            ("DocumentRejected", Some(0), Some("poi/blocks/bad.json")),
            ("InvalidValue", Some(0), Some("poi/blocks/bad.json")),
            ("PoiNoBlocks", Some(1), Some("poi/blocks/empty.json")),
        ]
    );
}
