//! `OneOrMany`: the one form and the array form read and write as written, and the walks
//! step into the array form only.

mod common;

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::schema::{self, Schema, SchemaCx};
use bedrock_addon_core::shared::EventTrigger;
use bedrock_addon_core::versioning::Reading;
use bedrock_addon_shared_shapes::OneOrMany;
use common::round_trip;
use facet::Facet;

#[test]
fn the_one_form_and_the_array_form_are_kept() {
    let one: OneOrMany<String> = round_trip(r#""a""#);
    assert_eq!(one, OneOrMany::One("a".to_owned()));
    assert_eq!(one.as_slice(), ["a"]);
    assert!(!one.is_many());
    let many: OneOrMany<String> = round_trip(r#"["a", "b"]"#);
    assert_eq!(many.as_slice(), ["a", "b"]);
    assert_eq!(many.len(), 2);
    // One value in an array stays an array.
    let single: OneOrMany<String> = round_trip(r#"["a"]"#);
    assert!(single.is_many());
    let empty: OneOrMany<String> = round_trip("[]");
    assert!(empty.is_empty());
    assert_eq!(single.clone().into_vec(), ["a"]);
    assert_eq!((&many).into_iter().count(), 2);
    assert_eq!(OneOrMany::from(vec![1, 2]), OneOrMany::Many(vec![1, 2]));
    assert_eq!(OneOrMany::<i32>::default(), OneOrMany::Many(Vec::new()));
    assert_eq!(OneOrMany::One(3).into_iter().sum::<i32>(), 3);
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Bounded {
    #[facet(schema::max = 1)]
    level: i32,
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Holder {
    entries: OneOrMany<Bounded>,
    on_hit: Option<OneOrMany<EventTrigger>>,
}

/// The paths of the findings of `holder`'s check.
fn checked(holder: &mut Holder) -> Vec<String> {
    let mut found = Diagnostics::default();
    holder.check(&mut SchemaCx::new(
        Reading::DOCUMENT,
        JsonPath::new(),
        &mut found,
    ));
    found
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect()
}

#[test]
fn the_check_steps_into_the_array_form_only() {
    let mut one: Holder = round_trip(r#"{"entries": {"level": 2}}"#);
    assert_eq!(checked(&mut one), ["entries/level"]);
    let mut many: Holder = round_trip(r#"{"entries": [{"level": 0}, {"level": 2}]}"#);
    assert_eq!(checked(&mut many), ["entries/1/level"]);
    assert_eq!(Holder::triggers(), ["on_hit", "on_hit[]"]);
}
