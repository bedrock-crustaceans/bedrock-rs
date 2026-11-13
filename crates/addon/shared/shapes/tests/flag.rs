//! `TrueOnly`: `true` reads and writes back, `false` is refused, and an absent member is
//! `None`.

use bedrock_addon_test_support::value as common;

use bedrock_addon_shared_shapes::{NotTrue, TrueOnly};
use common::{read, round_trip};
use facet::Facet;

#[derive(Debug, PartialEq, Facet)]
struct Holder {
    #[facet(default)]
    flag: Option<TrueOnly>,
}

#[test]
fn true_is_the_one_value() {
    assert_eq!(round_trip::<TrueOnly>("true"), TrueOnly);
    assert!(read::<TrueOnly>("false").is_err());
    assert!(read::<TrueOnly>("1").is_err());
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
