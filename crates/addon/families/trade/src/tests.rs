//! Unit tests of the module's private parts: the texts' arity, and where a failed typed parse
//! is reported.

use super::read::scalar_member;
use super::texts;
use bedrock_addon_core::diagnostic::JsonPathSegment;

/// `scalar_member` of the path `segments`: an all-digit segment is an array index, any other a
/// key.
fn at(segments: &[&str]) -> Option<usize> {
    let path: Vec<JsonPathSegment<'_>> = segments
        .iter()
        .map(|segment| match segment.parse() {
            Ok(index) => JsonPathSegment::Index(index),
            Err(_) => JsonPathSegment::Key(segment),
        })
        .collect();
    scalar_member(&path)
}

/// A finding below a scalar member is the member's: the member is found by its place in the
/// slice's struct, not by its name anywhere in the path.
#[test]
fn a_scalar_member_is_found_by_its_place_in_the_path() {
    // The scalar members where the typed structs have them.
    assert_eq!(at(&["3", "total_exp_required", "0"]), Some(1));
    assert_eq!(at(&["0", "groups", "2", "num_to_select", "0"]), Some(3));
    for member in ["trader_exp", "max_uses", "reward_exp", "weight"] {
        assert_eq!(
            at(&["0", "groups", "0", "trades", "1", member, "0", "0"]),
            Some(5),
            "{member}"
        );
    }
    for member in ["item", "price_multiplier"] {
        assert_eq!(at(&[member, "key"]), Some(0), "{member}");
    }
    // A segment of the same name elsewhere is not one.
    assert_eq!(at(&["filters", "all_of", "0", "item", "0"]), None);
    assert_eq!(at(&["functions", "0", "weight", "0"]), None);
    assert_eq!(
        at(&["0", "groups", "0", "trades", "1", "wants", "max"]),
        None
    );
    assert_eq!(at(&["0", "groups", "0", "weight", "0"]), None);
    assert_eq!(
        at(&["0", "groups", "0", "trades", "1", "num_to_select"]),
        None
    );
    assert_eq!(at(&["0", "item", "0"]), None);
    assert_eq!(at(&["total_exp_required", "0"]), None);
    assert_eq!(at(&["0", "sets", "0", "num_to_select", "0"]), None);
    assert_eq!(at(&["quantity", "min", "0"]), None);
    // A quantity's bounds are read from their own slices, so no typed path ends below one.
    assert_eq!(at(&["min", "0"]), None);
    // A path that ends before the member's place has none.
    assert_eq!(at(&["0", "groups", "0", "trades"]), None);
    assert_eq!(at(&[]), None);
    // An array-index place takes an index only: a key spelled as digits is not one, and a key
    // place takes no index (review minor d).
    assert_eq!(
        scalar_member(&[
            JsonPathSegment::Key("0"),
            JsonPathSegment::Key("total_exp_required"),
            JsonPathSegment::Index(0),
        ]),
        None
    );
    assert_eq!(
        scalar_member(&[
            JsonPathSegment::Index(0),
            JsonPathSegment::Key("total_exp_required"),
            JsonPathSegment::Index(0),
        ]),
        Some(1)
    );
    assert_eq!(
        scalar_member(&[
            JsonPathSegment::Index(0),
            JsonPathSegment::Index(7),
            JsonPathSegment::Index(0),
            JsonPathSegment::Key("num_to_select"),
        ]),
        None
    );
}

#[test]
fn every_formatted_text_has_one_placeholder_per_argument() {
    for (template, arity) in texts::FORMATTED {
        assert_eq!(template.matches("{}").count(), *arity, "{template}");
    }
}

#[test]
fn equal_quantities_hash_alike() {
    use std::collections::HashSet;
    let quantities: HashSet<super::Quantity> =
        [super::Quantity::Exact(3), super::Quantity::Exact(3)]
            .into_iter()
            .collect();
    assert_eq!(quantities.len(), 1);
}
