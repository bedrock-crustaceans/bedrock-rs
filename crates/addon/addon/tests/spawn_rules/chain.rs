//! The two single-struct chains: contiguity, the range models' selection, floors and ceilings, no
//! beta; and the history enums' own-version write.

use bedrock_addon::families::spawn_rules::{
    SimplifiedFamily, SimplifiedHistory, SpawnFamily, SpawnHistory,
};
use bedrock_addon::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput};
use bedrock_addon::versioning::{
    Family, RangeModel, Selection, Single, VersionChain, check_chain, select,
};

use super::common::{context, document, experiment, legacy, loads, rejected, standard};

fn selects<F: Family>(version: &str) -> Selection {
    select::<F>(&FormatVersionInput::parse(version).expect("a version"))
}

fn engine() -> String {
    ENGINE_VERSION.to_string()
}

#[test]
fn every_chain_is_one_struct_without_beta() {
    check_chain::<SpawnFamily>().unwrap();
    check_chain::<SimplifiedFamily>().unwrap();
    assert_eq!(
        <SpawnHistory as VersionChain<SpawnFamily>>::VERSIONS,
        [FormatVersion::new(1, 8, 0)]
    );
    assert_eq!(
        <SimplifiedHistory as VersionChain<SimplifiedFamily>>::VERSIONS,
        [FormatVersion::new(1, 21, 60)]
    );
    const { assert!(!<SpawnHistory as VersionChain<SpawnFamily>>::HAS_BETA) };
    const { assert!(!<SimplifiedHistory as VersionChain<SimplifiedFamily>>::HAS_BETA) };
}

#[test]
fn spawn_is_closed_to_the_engine() {
    assert_eq!(SpawnFamily::RANGE_MODEL, RangeModel::ClosedToEngine);
    assert_eq!(selects::<SpawnFamily>("1.7.9"), Selection::BelowFloor);
    assert_eq!(selects::<SpawnFamily>("1.8.0"), Selection::Struct(0));
    assert_eq!(selects::<SpawnFamily>("1.21.60"), Selection::Struct(0));
    assert_eq!(selects::<SpawnFamily>(&engine()), Selection::Struct(0));
    assert_eq!(selects::<SpawnFamily>("99.0.0"), Selection::Unsupported);
}

#[test]
fn spawn_simplified_is_half_open_to_the_engine() {
    assert_eq!(SimplifiedFamily::RANGE_MODEL, RangeModel::HalfOpenToEngine);
    assert_eq!(
        selects::<SimplifiedFamily>("1.21.50"),
        Selection::BelowFloor
    );
    assert_eq!(selects::<SimplifiedFamily>("1.21.60"), Selection::Struct(0));
    assert_eq!(
        selects::<SimplifiedFamily>(&engine()),
        Selection::Unsupported
    );
}

#[test]
fn a_beta_document_is_refused() {
    // No beta struct in either spawn-rules family (`Beta = Infallible`).
    let text = document("beta", r#"{"description": {"identifier": "x:mob"}}"#);
    for cx in [context(), experiment()] {
        let d = rejected(&text, &cx);
        assert!(d.has_errors());
    }
}

#[test]
fn the_history_enum_writes_its_own_version() {
    let document = loads(
        &legacy(r#"{"minecraft:weight": {"default": 3}}"#),
        &context(),
    );
    let any: SpawnHistory = Single(standard(&document).clone());
    assert_eq!(any.parsed_version(), FormatVersion::new(1, 8, 0));
    let text = any.to_json().unwrap();
    assert!(
        text.starts_with("{\n  \"format_version\": \"1.8.0\""),
        "{text}"
    );
}
