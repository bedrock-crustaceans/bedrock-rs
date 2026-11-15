//! The two choice members (`scatter_chance`, a coordinate) as a reading that keeps no unknown
//! member takes them (the scatter feature: the game's conversion of the whole member) and as a
//! reading that keeps them (a feature rule's: the struct form member by member).

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::parse::{Options, parse};
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::schema::FromJson;
use bedrock_addon_core::versioning::{BeyondWidth, Choices, Reading, Scalars};
use bedrock_addon_shared_scatter::{CoordinateRange, ScatterChance, ScatterFraction};

const KEEPING: Reading = Reading::DOCUMENT
    .with_scalars(Scalars::Strict)
    .with_beyond_width(BeyondWidth::Error)
    .with_keeps_unknown(true);

const NOT_KEEPING: Reading = KEEPING
    .with_keeps_unknown(false)
    .with_choices(Choices::Whole);

/// Reads `text` as `T` under `reading`: the value and each finding as `severity path code`.
fn read<T: FromJson>(text: &str, reading: Reading) -> (Option<T>, Vec<String>) {
    let parsed = parse(text, Options::default()).expect("JSON");
    let node = Node::parsed(&parsed, JsonPath::of_keys(["x"]));
    let mut found = Diagnostics::default();
    let mut read = Read::new(reading, &mut found);
    let value = T::from_json(&node, &mut read);
    let told = found
        .iter()
        .map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code.id()))
        .collect();
    (value, told)
}

#[test]
fn a_number_beyond_f32_is_clamped_and_told_where_the_reading_keeps_no_unknown_member() {
    let (value, told) = read::<CoordinateRange>("1e39", NOT_KEEPING);
    assert_eq!(
        value,
        Some(CoordinateRange::Single(Molang::Const(f32::MAX)))
    );
    assert_eq!(told, ["Info x core/invalid_value"]);
    // Each bound of an extent is told at its own index.
    let (value, told) = read::<CoordinateRange>(
        r#"{"distribution": "uniform", "extent": [-4e38, 1e39]}"#,
        NOT_KEEPING,
    );
    assert!(matches!(value, Some(CoordinateRange::Object(_))));
    assert_eq!(
        told,
        [
            "Info x/extent/0 core/invalid_value",
            "Info x/extent/1 core/invalid_value"
        ]
    );
    let (value, told) = read::<ScatterChance>("-1e39", NOT_KEEPING);
    assert_eq!(value, Some(ScatterChance::Percent(Molang::Const(f32::MIN))));
    assert_eq!(told, ["Info x core/invalid_value"]);
}

#[test]
fn a_number_beyond_f32_is_refused_where_the_reading_keeps_unknown_members() {
    let (value, told) = read::<CoordinateRange>("1e39", KEEPING);
    assert_eq!(value, None);
    assert_eq!(told, ["Error x core/invalid_type"]);
}

#[test]
fn a_struct_form_with_a_member_it_does_not_take_is_no_form_where_nothing_is_kept() {
    let text = r#"{"numerator": 1, "denominator": 2, "extra": 1}"#;
    let (value, told) = read::<ScatterChance>(text, NOT_KEEPING);
    assert_eq!(value, None);
    assert_eq!(told, ["Error x core/invalid_type"]);
    // A reading that keeps unknown members reads the form and tells the member.
    let (value, told) = read::<ScatterChance>(text, KEEPING);
    assert_eq!(
        value,
        Some(ScatterChance::Fraction(ScatterFraction {
            numerator: 1,
            denominator: 2
        }))
    );
    assert_eq!(told.len(), 1);
    assert!(
        told[0].starts_with(&format!("{:?} x/extra", Severity::Error)),
        "{told:?}"
    );
}
