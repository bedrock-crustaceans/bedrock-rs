//! `StrictBool`, the one strict wrapper left: a bool whatever the slice's scalar rule says, a
//! `true` or `false` token and nothing else.

use bedrock_addon_core::json::{StrictBool, from_str_jsonc};
use bedrock_addon_core::schema::read_value;

/// `json` read as the field.
fn read(json: &str) -> Option<StrictBool> {
    read_value::<StrictBool>(&from_str_jsonc(json).unwrap())
}

const SHAPES: [&str; 6] = ["1", "1.0", "\"-64\"", "true", "null", "3000000000"];

#[test]
fn strict_bool_takes_only_true_and_false() {
    let accepted: Vec<bool> = SHAPES.iter().map(|json| read(json).is_some()).collect();
    assert_eq!(accepted, [false, false, false, true, false, false]);
    assert!(read("\"true\"").is_none());
    assert_eq!(read("false"), Some(StrictBool(false)));
}
