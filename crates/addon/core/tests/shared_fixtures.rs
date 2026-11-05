//! The family fixture rows whose members are shared types, copied under
//! `tests/fixtures/shared/<family>/<row>.json` with only the members that hold the shared type.
//! Each test pins what the shared type does for its row: the value read, the form written back
//! and the type's own findings. What the row asserts beyond that (Molang compile versions,
//! dropping a bad entry, family defaults and constraints) is the family's and is not tested
//! here.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "tests read fixtures and call facet-json directly"
)]
#[path = "shared/common.rs"]
mod common;

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::shared::{Filter, FilterScalar, FilterSubject};
use common::{compact, normalised};
use facet::Facet;

/// Reads a fixture the way a family reads its file, and requires the written value to equal the
/// fixture.
fn read_fixture<T: for<'a> Facet<'a>>(text: &str) -> T {
    let value: T = facet_json::from_str_borrowed_jsonc(text).expect("the fixture reads");
    assert_eq!(compact(&value), normalised(text), "written form differs");
    value
}

#[derive(Facet)]
struct BiomeFilter {
    #[facet(rename = "minecraft:biome_filter")]
    biome_filter: Filter,
}

#[test]
fn sg20_biome_filter() {
    let row: BiomeFilter = read_fixture(include_str!(
        "fixtures/shared/spawn_groups/sg20_biome_filter.json"
    ));
    let Filter::Test(test) = &row.biome_filter else {
        panic!("not a test")
    };
    assert_eq!(test.test, "has_biome_tag");
    assert_eq!(test.operator.as_deref(), Some("=="));
    assert_eq!(test.value, Some(FilterScalar::String("animal".into())));
    assert!(row.biome_filter.check(&JsonPath::new()).is_empty());
}

#[derive(Facet)]
struct Condition {
    weight: u32,
    #[facet(default)]
    mob_event_filter: Option<String>,
    #[facet(default)]
    biome_filter: Filter,
}

#[derive(Facet)]
struct Conditions {
    conditions: Vec<Condition>,
}

#[test]
fn s12_simplified_defaults() {
    let text = include_str!("fixtures/shared/spawn/s12_simplified_defaults.json");
    let row: Conditions = facet_json::from_str_borrowed_jsonc(text).unwrap();
    // An absent `biome_filter` is the empty group: unfiltered.
    assert_eq!(row.conditions[0].weight, 99);
    assert_eq!(row.conditions[0].biome_filter, Filter::default());
}

#[test]
fn s13_simplified_multiple_conditions() {
    let text = include_str!("fixtures/shared/spawn/s13_simplified_multiple_conditions.json");
    let row: Conditions = facet_json::from_str_borrowed_jsonc(text).unwrap();
    let second = &row.conditions[1];
    assert_eq!(second.mob_event_filter.as_deref(), Some("mob:filter"));
    let Filter::Test(leaf) = &second.biome_filter else {
        panic!("not a test")
    };
    assert_eq!(
        (leaf.test.as_str(), leaf.value.as_ref()),
        (
            "has_biome_tag",
            Some(&FilterScalar::String("plains".into()))
        )
    );
    assert!(second.biome_filter.check(&JsonPath::new()).is_empty());
}

#[test]
fn r01_1_13_0_loads() {
    let row: BiomeFilter = read_fixture(include_str!(
        "fixtures/shared/feature_rules/r01_1_13_0_loads.json"
    ));
    let Filter::List(list) = &row.biome_filter else {
        panic!("not a list")
    };
    assert!(
        matches!(&list[0], Filter::Test(t) if t.subject.unwrap_or_default() == FilterSubject::Self_)
    );
    assert!(matches!(&list[1], Filter::Group(g) if g.any_of.is_some()));
    assert!(row.biome_filter.check(&JsonPath::new()).is_empty());
}

#[test]
fn jig04_vanilla_test_inputs() {
    // `biome_filters` defaults to an empty filter group;
    // vanilla files write it as `{AND: null, …}`, which reads as the empty group.
    #[derive(Facet)]
    struct Row {
        biome_filters: Filter,
    }
    let text = include_str!("fixtures/shared/jigsaw/jig04_vanilla_test_inputs.json");
    let row: Row = facet_json::from_str_borrowed_jsonc(text).unwrap();
    assert_eq!(row.biome_filters, Filter::default());
}
