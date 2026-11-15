//! `TrueOnly`: `true` reads and writes back, `false` is refused, and an absent member is
//! `None`.

use bedrock_addon_test_support::value as common;

use bedrock_addon_core::schema::Schema;
use bedrock_addon_shared_shapes::{NotTrue, TrueOnly};
use common::{read, round_trip};
use facet::Facet;

#[derive(Debug, PartialEq, Facet, Schema)]
struct Holder {
    flag: Option<TrueOnly>,
}

#[test]
fn true_is_the_one_value() {
    assert_eq!(round_trip::<TrueOnly>("true"), TrueOnly);
    assert!(read::<TrueOnly>("false").is_none());
    assert!(read::<TrueOnly>("1").is_none());
    assert_eq!(TrueOnly::try_from(false), Err(NotTrue));
    assert!(bool::from(TrueOnly));
}

#[test]
fn an_absent_flag_is_none() {
    let holder: Holder = round_trip("{}");
    assert_eq!(holder.flag, None);
    let holder: Holder = round_trip(r#"{"flag": true}"#);
    assert_eq!(holder.flag, Some(TrueOnly));
}

#[test]
fn the_derived_read_takes_true_and_tells_false_as_a_value_not_allowed() {
    use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
    use bedrock_addon_core::json::kit::{Node, Read};
    use bedrock_addon_core::json::{Value, from_str_jsonc};
    use bedrock_addon_core::schema::FromJson;
    use bedrock_addon_core::versioning::{Reading, Scalars};

    let read = |text: &str| {
        let json: Value = from_str_jsonc(text).unwrap();
        let mut found = Diagnostics::default();
        let strict = Reading::DOCUMENT.with_scalars(Scalars::Strict);
        let mut read = Read::new(strict, &mut found);
        let flag = TrueOnly::from_json(&Node::new(&json, JsonPath::of_keys(["flag"])), &mut read);
        drop(read);
        let told: Vec<(Severity, &'static str)> =
            found.iter().map(|f| (f.severity, f.code.id())).collect();
        (flag, told)
    };
    assert_eq!(read("true"), (Some(TrueOnly), vec![]));
    assert_eq!(
        read("false"),
        (None, vec![(Severity::Error, "core/value_not_allowed")])
    );
    assert_eq!(
        read("\"true\""),
        (None, vec![(Severity::Error, "core/invalid_type")])
    );
}
