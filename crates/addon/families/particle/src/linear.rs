//! What the cost tests of the particle modules share: the builders of their inputs and the
//! shared protocol (`bedrock_addon_core::test_cost`) for a function that borrows a JSON value.
//!
//! Only the call of the function is timed: the input is built before the clock starts and the
//! output is dropped after it stops, so that a walk that is quadratic in the entries is the
//! whole of what the ratio shows.

use super::derived::kit_read;
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::molang::RawVersion;
use bedrock_addon_core::test_cost::assert_linear_at;

/// The smaller size of the family's cost tests; the larger is four times it (100 000).
pub(crate) const SINGLE: usize = 25_000;

/// The shared protocol for a reader, which borrows its input: `make` builds the value of each
/// size once, before anything is measured, and `read` is all that is timed.
#[track_caller]
pub(crate) fn assert_reads_linear<O>(
    what: &str,
    make: impl Fn(usize) -> Value,
    read: impl FnMut(&Value, usize) -> O,
) {
    assert_reads_linear_at(what, SINGLE, make, read);
}

/// [`assert_reads_linear`] at `n` entries against four times `n`, for a reader too fast to be
/// measured at [`SINGLE`].
#[track_caller]
pub(crate) fn assert_reads_linear_at<O>(
    what: &str,
    n: usize,
    make: impl Fn(usize) -> Value,
    mut read: impl FnMut(&Value, usize) -> O,
) {
    assert_linear_at(
        what,
        n,
        |count| (make(count), count),
        |(input, count)| read(input, *count),
    );
}

/// `text` as a JSON value: what a reader is handed.
pub(crate) fn value(text: &str) -> Value {
    bedrock_addon_core::json::from_str_jsonc(text).expect("the test's JSON")
}

/// An object of `count` members `"<prefix><i>": <entry>`.
pub(crate) fn object_of(count: usize, prefix: &str, entry: &str) -> Value {
    let members: Vec<String> = (0..count)
        .map(|i| format!(r#""{prefix}{i}": {entry}"#))
        .collect();
    value(&format!("{{{}}}", members.join(", ")))
}

/// An array of `count` elements `entry`.
pub(crate) fn array_of(count: usize, entry: &str) -> Value {
    value(&format!("[{}]", vec![entry; count].join(", ")))
}

/// The two eras, named, and whether each is the legacy parser's.
pub(crate) const ERAS: [(&str, bool); 2] = [("legacy", true), ("1.20.80", false)];

/// `value` as the node at the root of a read.
pub(crate) fn root(value: &Value) -> Node<'_> {
    Node::new(value, JsonPath::new())
}

/// Runs `read` with a read of the era (`legacy` the legacy parser's) and returns what it read
/// with the findings.
pub(crate) fn reading<T>(legacy: bool, read: impl FnOnce(&mut Read<'_>) -> T) -> (T, Diagnostics) {
    let mut found = Diagnostics::default();
    let mut r = kit_read(legacy, RawVersion(13), Severity::Warning, &mut found);
    let value = read(&mut r);
    r.finish();
    drop(r);
    (value, found)
}
