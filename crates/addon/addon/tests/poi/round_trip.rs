//! Write-back at the Value level: `parse(write(parse(x))) == parse(x)` for
//! every row document that loads, a stable rewrite, the members in vanilla's bind order, and
//! no member the author left out; and the as-written parse

#![allow(
    clippy::disallowed_methods,
    reason = "calls facet-json directly: parses a fixture or written text into a json Value to compare it; not yet moved to the door"
)]

use std::fmt::Debug;

use bedrock_addon::document::{Content, JsonDocument};
use bedrock_addon::families::poi::history::{upgrade_tag, upgrade_type};
use bedrock_addon::families::poi::{PoiTag, PoiType};
use bedrock_addon::id::{BlockId, PoiTagId, PoiTypeId};
use bedrock_addon::json::{Value, WriteStyle};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::FormatVersion;
use bedrock_addon::versioning::Single;

use super::common::{block_document, context, fixture, loads, tag_of};

/// Loads the fixture file `rest`, writes it, reads the text back and requires the same value,
/// the same findings and a stable rewrite; the written text.
#[track_caller]
fn round_trip<T: Content + Debug + PartialEq>(rest: &str) -> String {
    let first = loads::<T>(&fixture(rest));
    let written = first.to_json().expect("written");
    let second = loads::<T>(&written);
    assert_eq!(first.value(), second.value(), "{rest}");
    assert_eq!(
        first.diagnostics(),
        second.diagnostics(),
        "{rest}: the rewrite loads as the file did"
    );
    assert_eq!(
        second.to_json().unwrap(),
        written,
        "{rest}: a stable rewrite"
    );
    written
}

/// The keys of the object `key` of the written document `text`, in order.
fn payload_keys(text: &str, key: &str) -> Vec<String> {
    let json: Value = facet_json::from_str(text).expect("the written text is JSON");
    let root = json.as_object().expect("an object");
    let root_keys: Vec<&str> = root.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(root_keys, ["format_version", key], "the envelope");
    assert_eq!(
        root.get("format_version")
            .and_then(Value::as_string)
            .map(std::ops::Deref::deref),
        Some("1.26.40")
    );
    root.get(key)
        .and_then(Value::as_object)
        .expect("the payload")
        .iter()
        .map(|(k, _)| k.as_str().to_owned())
        .collect()
}

#[test]
fn every_row_document_that_loads_round_trips() {
    for rest in [
        "poi01_block_minimal/poi/blocks/forge.json",
        "poi02_block_full/poi/blocks/guild_hall.json",
        "poi03_vanilla_meeting/poi/blocks/meeting.json",
        "poi18_empty_blocks/poi/blocks/forge.json",
        "poi31_unknown_block/poi/blocks/forge.json",
        "poi33_type_tag_without_document/poi/blocks/forge.json",
        "poi30_vanilla_1_26_20_set/poi/blocks/mason.json",
        "poi30_vanilla_1_26_20_set/poi/blocks/beehive.json",
    ] {
        round_trip::<PoiType>(rest);
    }
    for rest in [
        "poi04_tag_minimal/poi/tags/civic.json",
        "poi05_tag_full/poi/tags/acquirable_job_site.json",
        "poi32_unknown_tag_reference/poi/tags/a.json",
        "poi34_tag_cycle/poi/tags/t1.json",
        "poi41_cycle_keeps_direct_types/poi/tags/t1.json",
    ] {
        round_trip::<PoiTag>(rest);
    }
}

#[test]
fn the_fixture_files_rewrite_byte_for_byte() {
    // The fixtures are written the way the writer writes: two spaces, bind order.
    for rest in [
        "poi01_block_minimal/poi/blocks/forge.json",
        "poi02_block_full/poi/blocks/guild_hall.json",
    ] {
        assert_eq!(
            round_trip::<PoiType>(rest).trim_end(),
            fixture(rest).trim_end(),
            "{rest}"
        );
    }
    let rest = "poi05_tag_full/poi/tags/acquirable_job_site.json";
    assert_eq!(
        round_trip::<PoiTag>(rest).trim_end(),
        fixture(rest).trim_end()
    );
}

#[test]
fn members_the_author_left_out_are_not_written() {
    let written = round_trip::<PoiType>("poi01_block_minimal/poi/blocks/forge.json");
    assert_eq!(
        payload_keys(&written, "minecraft:poi_block"),
        ["description", "blocks"]
    );
    let written = round_trip::<PoiTag>("poi04_tag_minimal/poi/tags/civic.json");
    assert_eq!(payload_keys(&written, "minecraft:poi_tag"), ["description"]);
    // A default the author did write stays.
    let text = block_document(
        "1.26.40",
        r#"{"description": {"identifier": "test:forge"}, "tickets": 0, "usable_distance": 1, "blocks": [], "tags": []}"#,
    );
    let explicit = loads::<PoiType>(&text).to_json().unwrap();
    assert_eq!(
        payload_keys(&explicit, "minecraft:poi_block"),
        [
            "description",
            "tickets",
            "usable_distance",
            "blocks",
            "tags"
        ]
    );
    let empty = loads::<PoiTag>(&tag_of("test:civic", r#", "parents": [], "children": []"#))
        .to_json()
        .unwrap();
    assert_eq!(
        payload_keys(&empty, "minecraft:poi_tag"),
        ["description", "children", "parents"]
    );
}

#[test]
fn a_built_value_is_written_in_bind_order_at_1_26_40() {
    // Members out of order in, vanilla's bind order (the schema ordinals) out.
    let shuffled = block_document(
        "1.26.50",
        r#"{"tags": ["test:civic"], "blocks": ["minecraft:bell"], "usable_distance": 4, "tickets": 3, "description": {"identifier": "test:hall"}}"#,
    );
    let hall = loads::<PoiType>(&shuffled);
    assert_eq!(hall.version().written, Some(FormatVersion::new(1, 26, 50)));
    let built = PoiType {
        tickets: Some(3),
        usable_distance: Some(4),
        tags: Some(vec![PoiTagId::from_static("test:civic")]),
        ..PoiType::new(
            PoiTypeId::from_static("test:hall"),
            vec![BlockId::from_static("minecraft:bell")],
        )
    };
    assert_eq!(*hall.value(), built);
    let written = built.to_json().unwrap();
    assert_eq!(
        payload_keys(&written, "minecraft:poi_block"),
        [
            "description",
            "tickets",
            "usable_distance",
            "blocks",
            "tags"
        ]
    );
    assert_eq!(*loads::<PoiType>(&written).value(), built);
    let tag = PoiTag {
        parents: Some(vec![PoiTagId::from_static("test:town")]),
        children: Some(vec![PoiTagId::from_static("test:job")]),
        ..PoiTag::new(PoiTagId::from_static("test:civic"))
    };
    let written = tag.to_json().unwrap();
    assert_eq!(
        payload_keys(&written, "minecraft:poi_tag"),
        ["description", "children", "parents"]
    );
    assert_eq!(*loads::<PoiTag>(&written).value(), tag);
}

#[test]
fn write_json_writes_what_to_json_gives() {
    let forge = PoiType::new(
        PoiTypeId::from_static("test:forge"),
        vec![BlockId::from_static("minecraft:anvil")],
    );
    let mut out = Vec::new();
    forge.write_json(&mut out, WriteStyle::default()).unwrap();
    assert_eq!(String::from_utf8(out).unwrap(), forge.to_json().unwrap());
    let civic = PoiTag::new(PoiTagId::from_static("test:civic"));
    let mut out = Vec::new();
    civic.write_json(&mut out, WriteStyle::default()).unwrap();
    assert_eq!(String::from_utf8(out).unwrap(), civic.to_json().unwrap());
}

#[test]
fn what_the_load_removed_is_not_written_back() {
    // B11: the repeated block; T5: the edge to itself (its member stays, empty).
    let written =
        round_trip_lossy::<PoiType>("poi19_duplicate_block_in_list/poi/blocks/forge.json");
    assert_eq!(written.matches("minecraft:bell").count(), 1);
    let written = round_trip_lossy::<PoiTag>("poi21_tag_self_parent/poi/tags/a.json");
    assert_eq!(
        payload_keys(&written, "minecraft:poi_tag"),
        ["description", "parents"]
    );
    assert_eq!(written.matches("test:a").count(), 1, "only the identifier");
}

/// [`round_trip`] for a document whose load reports something: the rewrite loads cleanly.
#[track_caller]
fn round_trip_lossy<T: Content + Debug + PartialEq>(rest: &str) -> String {
    let first = loads::<T>(&fixture(rest));
    assert!(!first.diagnostics().is_empty(), "{rest}: the load reports");
    let written = first.to_json().expect("written");
    let second = loads::<T>(&written);
    assert_eq!(first.value(), second.value(), "{rest}");
    assert!(
        second.diagnostics().is_empty(),
        "{rest}: the rewrite is clean"
    );
    written
}

#[test]
fn parse_as_written_then_upgrade_is_the_load() {
    let path = PackPath::new("poi/blocks/guild_hall.json").unwrap();
    let text = fixture("poi02_block_full/poi/blocks/guild_hall.json");
    let cx = context();
    let as_written = PoiType::parse_as_written(&path, text.as_bytes(), &cx).expect("parses");
    let Single(inner) = as_written.value();
    let loaded = PoiType::parse(&path, text.as_bytes(), &cx).expect("loads");
    assert_eq!(inner, loaded.value());
    let upgraded = upgrade_type(as_written, &cx).expect("upgrades");
    assert_eq!(upgraded, loaded);

    let text = fixture("poi21_tag_self_parent/poi/tags/a.json");
    let as_written = PoiTag::parse_as_written(&path, text.as_bytes(), &cx).expect("parses");
    let Single(inner) = as_written.value();
    assert_eq!(
        inner.parents(),
        [PoiTagId::from_static("test:a")],
        "as written: the validation has not run"
    );
    let upgraded = upgrade_tag(as_written, &cx).expect("upgrades");
    assert_eq!(
        upgraded,
        PoiTag::parse(&path, text.as_bytes(), &cx).expect("loads")
    );
    assert!(upgraded.parents().is_empty());

    // A rejected file is rejected as written too.
    let bad = fixture("poi06_below_floor/poi/blocks/forge.json");
    assert!(PoiType::parse_as_written(&path, bad.as_bytes(), &cx).is_err());
}
