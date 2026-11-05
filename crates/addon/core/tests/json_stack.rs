//! The readers of `json` on a small stack: a document nested up to
//! `MAX_JSON_DEPTH` is read on a thread with a megabyte of stack, the stack of a spawned
//! thread on some hosts, and a level more is the depth error, in a debug build as in any
//! other; into an untyped value, into raw members and into a typed target, one with an untyped
//! member or one that holds itself included.
//!
//! A stack that overflows aborts the process, so each case runs in a child process and the
//! test looks at how it ended.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::process::{Command, Stdio};

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, MAX_JSON_DEPTH, Unrepresentable, UnsafeText, Value, Width, WriteStyle,
    check_depth, clamp_beyond_double, clamp_non_finite, clamp_tokens, from_slice_jsonc, from_str,
    from_str_borrowed_jsonc, from_str_jsonc, keep_unknown_members, last_wins, last_wins_except,
    parse_at, probe_envelope, scan, span_at, to_facet_json, to_json, trailing_content,
    unknown_members, write_json,
};
use facet::Facet;
use facet_format::DeserializeErrorKind;
use facet_json::{DeserializeError, RawJson};
use indexmap::IndexMap;

/// The stack of the thread the cases of the door run on.
const SMALL_STACK: usize = 1024 * 1024;

/// How a reader ended.
#[derive(Debug, PartialEq)]
enum Read {
    /// A value nested as deep as the text: checked level by level.
    Value(usize),
    /// The text was read; the reader gives no value to descend into.
    Text,
    /// The depth error, at the offset of the bracket past the limit.
    TooDeep(usize),
    /// Anything else.
    Other(String),
}

/// The depth of `value` along its first element or member, counted without recursion, and
/// the scalar at the bottom.
fn depth_of(value: &Value) -> (usize, &Value) {
    let (mut at, mut depth) = (value, 0);
    loop {
        let inner = match (at.as_array(), at.as_object()) {
            (Some(array), _) => array.iter().next(),
            (_, Some(object)) => object.iter().next().map(|(_, member)| member),
            _ => None,
        };
        match inner {
            Some(inner) => {
                at = inner;
                depth += 1;
            }
            None => return (depth, at),
        }
    }
}

fn read_of(result: Result<Value, DeserializeError>) -> Read {
    match result {
        Ok(value) => {
            let (depth, bottom) = depth_of(&value);
            if bottom.as_number().and_then(facet_value::VNumber::to_u64) == Some(7) {
                Read::Value(depth)
            } else {
                Read::Other(format!("the innermost value is {bottom:?}"))
            }
        }
        Err(error) => refusal_of(&error),
    }
}

fn refusal_of(error: &DeserializeError) -> Read {
    match (&error.kind, error.span) {
        (DeserializeErrorKind::UnexpectedChar { expected, .. }, Some(span))
            if expected.starts_with("at most") =>
        {
            Read::TooDeep(span.offset as usize)
        }
        _ => Read::Other(format!("{error:?}")),
    }
}

/// A struct with one level of its own above a raw member.
#[derive(Debug, Facet)]
struct Holder<'a> {
    #[facet(default)]
    raw: Option<RawJson<'a>>,
}

/// What a reader gives for a text within the limit and for one past it.
#[derive(Clone, Copy, PartialEq)]
enum Gives {
    /// The value, as deep as the text; the depth error.
    Value,
    /// Nothing to descend into; the depth error.
    Text,
    /// The value written and read again; nothing, since no helper refuses a text for its
    /// depth (the read beside it does).
    Helped,
}

/// A reader by name.
type Reader = (&'static str, Gives, fn(&str) -> Read);

/// Every entry point of the door, and the readers built on it.
const READERS: &[Reader] = &[
    ("from_str", Gives::Value, |text| {
        // Plain JSON has no comments: white space of their length keeps the offsets.
        read_of(from_str::<Value>(&text.replace("/**/", "    ")))
    }),
    ("from_str_jsonc", Gives::Value, |text| {
        read_of(from_str_jsonc::<Value>(text))
    }),
    ("from_str_borrowed_jsonc", Gives::Value, |text| {
        read_of(from_str_borrowed_jsonc::<Value>(text))
    }),
    ("from_slice_jsonc", Gives::Value, |text| {
        read_of(from_slice_jsonc::<Value>(text.as_bytes()))
    }),
    (
        "from_slice_jsonc of bytes that are no UTF-8",
        Gives::Value,
        |text| {
            // A comment holds the byte: the text is read.
            let mut bytes = b"// \xFF\n".to_vec();
            let before = bytes.len();
            bytes.extend_from_slice(text.as_bytes());
            match read_of(from_slice_jsonc::<Value>(&bytes)) {
                Read::TooDeep(offset) => Read::TooDeep(offset - before),
                other => other,
            }
        },
    ),
    (
        "parse_at into a value",
        Gives::Value,
        |text| match parse_at::<Value>(JsonSlice::new(text)) {
            Ok(value) => read_of(Ok(value)),
            Err(error) => refusal_of(&error.error),
        },
    ),
    ("parse_at into raw members", Gives::Value, |text| {
        // The root is the map's level: its members are raw slices of the text.
        if !text.starts_with('{') {
            return match parse_at::<Vec<RawJson<'_>>>(JsonSlice::new(text)) {
                Ok(list) => list.first().map_or(Read::Text, |raw| {
                    match read_of(from_str_jsonc::<Value>(raw.as_str())) {
                        Read::Value(depth) => Read::Value(depth + 1),
                        other => other,
                    }
                }),
                Err(error) => refusal_of(&error.error),
            };
        }
        match parse_at::<IndexMap<String, RawJson<'_>>>(JsonSlice::new(text)) {
            Ok(map) => map.values().next().map_or(Read::Text, |raw| {
                match read_of(from_str_jsonc::<Value>(raw.as_str())) {
                    Read::Value(depth) => Read::Value(depth + 1),
                    other => other,
                }
            }),
            Err(error) => refusal_of(&error.error),
        }
    }),
    (
        "parse_at into a struct of raw members",
        Gives::Value,
        |text| {
            // The text's inner value as the struct's `raw` member and as a member it does not
            // have (skipped): the struct is the level taken off.
            let Some(inner) = inner_of(text) else {
                return Read::Other("no inner value".to_owned());
            };
            let before = r#"{"raw": "#;
            let document = format!(r#"{before}{inner}, "unknown": {inner}}}"#);
            match parse_at::<Holder<'_>>(JsonSlice::new(&document)) {
                Ok(holder) => {
                    let raw = holder.raw.as_ref().map_or("", RawJson::as_str);
                    match read_of(from_str_jsonc::<Value>(raw)) {
                        Read::Value(depth) => Read::Value(depth + 1),
                        other => other,
                    }
                }
                Err(error) => match refusal_of(&error.error) {
                    // The offset the bracket has in the text.
                    Read::TooDeep(offset) => {
                        let taken = inner.as_ptr() as usize - text.as_ptr() as usize;
                        Read::TooDeep(offset - before.len() + taken)
                    }
                    other => other,
                },
            }
        },
    ),
    (TYPED, Gives::Value, |text| {
        // facet reads the level of the list or map and, by a call of itself for each level,
        // the value below it: the door gives it the stack for that.
        let below = |value: Option<&Value>| match value.map(depth_of) {
            Some((depth, bottom)) if bottom.as_number().is_some() => Read::Value(depth + 1),
            other => Read::Other(format!("{other:?}")),
        };
        if text.trim_start().starts_with('{') {
            match parse_at::<IndexMap<String, Value>>(JsonSlice::new(text)) {
                Ok(map) => below(map.values().next()),
                Err(error) => refusal_of(&error.error),
            }
        } else {
            match parse_at::<Vec<Value>>(JsonSlice::new(text)) {
                Ok(list) => below(list.first()),
                Err(error) => refusal_of(&error.error),
            }
        }
    }),
    ("probe_envelope", Gives::Text, |text| {
        match probe_envelope(JsonSlice::new(text)) {
            Ok(_) => Read::Text,
            Err(error) => match refusal_of(&error.error) {
                // A root that is no object is the envelope's own refusal.
                Read::Other(_) if !text.starts_with('{') => Read::Text,
                other => other,
            },
        }
    }),
    ("scan", Gives::Text, |text| {
        match scan(text.as_bytes(), true) {
            Ok(_) => Read::Text,
            Err(UnsafeText::TooDeep { offset }) => Read::TooDeep(offset),
            Err(other) => Read::Other(format!("{other:?}")),
        }
    }),
    ("check_depth", Gives::Text, |text| {
        match check_depth(text.as_bytes(), MAX_JSON_DEPTH) {
            Ok(()) => Read::Text,
            Err(deep) => Read::TooDeep(deep.offset),
        }
    }),
    (
        "the helpers over a text and its value",
        Gives::Helped,
        |text| {
            // What a family runs beside the read: none of them refuses a text for its depth,
            // and none may need more stack than the read.
            let slice = JsonSlice::new(text);
            let _ = last_wins(text);
            let _ = last_wins_except(text, &["a"]);
            let _ = trailing_content(text);
            let mut d = Diagnostics::default();
            let _ = clamp_beyond_double(slice, &JsonPath::new(), &mut d);
            let _ = clamp_tokens(
                slice,
                &|_| Some(Width::Float),
                &[],
                &JsonPath::new(),
                &mut d,
            );
            let Ok(mut value) = from_str_jsonc::<Value>(text) else {
                return Read::Text;
            };
            let _ = clamp_non_finite(&mut value, Some(slice));
            let _ = unknown_members(Value::SHAPE, &value);
            let _ = span_at(slice, &JsonPath::new());
            let copy = value.clone();
            if copy != value {
                return Read::Other("a copy differs".to_owned());
            }
            match to_json(&value) {
                Ok(written) => read_of(from_str_jsonc::<Value>(&written)),
                Err(error) => Read::Other(format!("{error:?}")),
            }
        },
    ),
];

/// The reader whose target has a level of its own above the values.
const TYPED: &str = "parse_at into a list or map of values";

/// `text` without its outermost bracket pair (and, for an object, its key).
fn inner_of(text: &str) -> Option<&str> {
    let text = text.trim();
    let inner = text.get(1..text.len().checked_sub(1)?)?.trim();
    Some(match inner.strip_prefix("\"a\"") {
        Some(member) => member.trim_start().strip_prefix(':')?.trim(),
        None => inner,
    })
}

/// A way of nesting by name: the text nested `levels` deep around the number 7, and the
/// offset of its first bracket past the limit (of its innermost bracket, within the limit).
type Shape = (&'static str, fn(usize) -> (String, usize));

/// The opening of level `level` (from 0) of a mix of both containers.
fn mixed(level: usize) -> &'static str {
    if level.is_multiple_of(2) {
        "{\"a\":"
    } else {
        "["
    }
}

fn nest(levels: usize, open: fn(usize) -> &'static str) -> (String, usize) {
    let mut text = String::new();
    let mut last = 0;
    for level in 0..levels {
        if level <= MAX_JSON_DEPTH {
            last = text.len();
        }
        text.push_str(open(level));
    }
    text.push('7');
    for level in (0..levels).rev() {
        text.push(if open(level).starts_with('[') {
            ']'
        } else {
            '}'
        });
    }
    (text, last)
}

const SHAPES: &[Shape] = &[
    ("arrays", |levels| nest(levels, |_| "[")),
    ("objects", |levels| nest(levels, |_| "{\"a\":")),
    ("objects and arrays", |levels| nest(levels, mixed)),
    ("arrays and objects", |levels| {
        nest(levels, |level| mixed(level + 1))
    }),
    ("arrays with a comment at every level", |levels| {
        // The door reads a copy of a text with comments: the same read, over the copy.
        let (text, last) = nest(levels, |_| "[/**/");
        (text, last)
    }),
    ("objects around an integer beyond 64 bits", |levels| {
        let (text, last) = nest(levels, |_| "{\"a\":");
        // A second member, so the first is still the one descended into.
        let wide = text.replacen('7', "7, \"b\": 18446744073709551616", 1);
        (wide, last)
    }),
];

/// A struct with an untyped member: facet reads the member by a call of itself for each
/// level of it.
#[derive(Clone, Debug, PartialEq, Facet)]
struct Sidecar {
    #[facet(default)]
    a: Option<Value>,
}

/// A struct that holds itself in a list: two levels of the text a turn.
#[derive(Clone, Debug, PartialEq, Facet)]
struct Tree {
    #[facet(default)]
    a: Option<Vec<Tree>>,
}

/// A struct that holds itself in a box: one level of the text a turn, and the most stack a
/// level of the three (an option, a pointer and a struct).
#[derive(Clone, Debug, PartialEq, Facet)]
struct Chain {
    #[facet(default)]
    a: Option<Box<Chain>>,
}

/// A struct that holds itself in a map.
#[derive(Clone, Debug, PartialEq, Facet)]
struct Named {
    #[facet(default)]
    a: Option<IndexMap<String, Option<Named>>>,
}

/// An untagged enum that holds itself in a list and in a struct: a filter's shape.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
enum Either {
    Many(Vec<Either>),
    One {
        #[facet(default)]
        a: Option<Box<Either>>,
    },
}

/// The entry points a typed target is read through.
const ENTRIES: &[&str] = &[
    "parse_at",
    "from_str_borrowed_jsonc",
    "from_str_jsonc",
    "from_str",
    "from_slice_jsonc",
    "parse_at and the walks beside it",
    "parse_at, written and read again",
];

/// `T` from `text` through entry point `entry`. With the last but one, what a family runs
/// beside the read is run too; with the last, the value is written and read again from what
/// was written.
fn through<T>(entry: usize, text: &'static str) -> Result<T, Read>
where
    T: Facet<'static> + Clone + PartialEq,
{
    let refused = |error: DeserializeError| refusal_of(&error);
    match entry {
        0 => parse_at::<T>(JsonSlice::new(text)).map_err(|error| refusal_of(&error.error)),
        1 => from_str_borrowed_jsonc::<T>(text).map_err(refused),
        2 => from_str_jsonc::<T>(text).map_err(refused),
        3 => from_str::<T>(text).map_err(refused),
        4 => from_slice_jsonc::<T>(text.as_bytes()).map_err(refused),
        5 => {
            let value =
                parse_at::<T>(JsonSlice::new(text)).map_err(|error| refusal_of(&error.error))?;
            let json = from_str_jsonc::<Value>(text).map_err(refused)?;
            let mut kept = bedrock_addon_core::document::Unknown::new();
            let unknown = keep_unknown_members(&value, &json, &mut kept);
            if !unknown.is_empty() || !unknown_members(T::SHAPE, &json).is_empty() {
                return Err(Read::Other(format!("unknown members: {unknown:?}")));
            }
            let copy = value.clone();
            if copy != value {
                return Err(Read::Other("a copy differs".to_owned()));
            }
            Ok(copy)
        }
        _ => {
            let value =
                parse_at::<T>(JsonSlice::new(text)).map_err(|error| refusal_of(&error.error))?;
            let written = to_json(&value).map_err(|error| Read::Other(format!("{error:?}")))?;
            let again = from_str_jsonc::<T>(&written).map_err(refused)?;
            if again != value {
                return Err(Read::Other("the value written reads as another".to_owned()));
            }
            Ok(again)
        }
    }
}

/// A typed case: the text nested as deep as asked with the offset of its bracket past the
/// limit, and how the read of it ended.
struct Typed {
    text: (String, usize),
    ended: Read,
}

/// The case of `T`: its text by `open` around `bottom`, read through `entry` and measured by
/// `depth`, which counts the levels of the value without a call for each.
fn typed<T>(
    levels: usize,
    entry: usize,
    open: fn(usize) -> &'static str,
    bottom: fn(usize) -> &'static str,
    depth: fn(&T) -> usize,
) -> Typed
where
    T: Facet<'static> + Clone + PartialEq,
{
    let (text, past) = nest(levels, open);
    let text = text.replacen('7', bottom(levels), 1);
    // The borrowing readers of an owned target want a text that outlives it.
    let kept: &'static str = Box::leak(text.clone().into_boxed_str());
    let ended = match through::<T>(entry, kept) {
        Ok(value) => Read::Value(depth(&value)),
        Err(ended) => ended,
    };
    Typed {
        text: (text, past),
        ended,
    }
}

/// A typed target by name: its case for a depth and an entry point.
type Target = (&'static str, fn(usize, usize) -> Typed);

const TARGETS: &[Target] = &[
    (
        "a struct with an untyped member of objects",
        |levels, entry| {
            typed::<Sidecar>(
                levels,
                entry,
                |_| "{\"a\":",
                |_| "7",
                |sidecar| 1 + sidecar.a.as_ref().map_or(0, |value| depth_of(value).0),
            )
        },
    ),
    (
        "a struct with an untyped member of arrays",
        |levels, entry| {
            typed::<Sidecar>(
                levels,
                entry,
                |level| if level == 0 { "{\"a\":" } else { "[" },
                |_| "7",
                |sidecar| 1 + sidecar.a.as_ref().map_or(0, |value| depth_of(value).0),
            )
        },
    ),
    ("a struct that holds itself in a list", |levels, entry| {
        typed::<Tree>(
            levels,
            entry,
            mixed,
            // The innermost container is a struct (its member absent) or an empty list.
            |levels| if levels.is_multiple_of(2) { "" } else { "null" },
            |tree| {
                let (mut at, mut depth) = (tree, 1);
                while let Some(list) = &at.a {
                    depth += 1;
                    match list.first() {
                        Some(inner) => {
                            at = inner;
                            depth += 1;
                        }
                        None => break,
                    }
                }
                depth
            },
        )
    }),
    ("a struct that holds itself in a box", |levels, entry| {
        typed::<Chain>(
            levels,
            entry,
            |_| "{\"a\":",
            |_| "null",
            |chain| {
                let (mut at, mut depth) = (chain, 1);
                while let Some(inner) = &at.a {
                    at = inner;
                    depth += 1;
                }
                depth
            },
        )
    }),
    ("a struct that holds itself in a map", |levels, entry| {
        typed::<Named>(
            levels,
            entry,
            |_| "{\"a\":",
            // The innermost object is a struct without its member or a map of none.
            |_| "null",
            |named| {
                // A struct, then the map in its member, then the struct in the map's.
                let (mut at, mut depth) = (named, 1);
                while let Some(map) = &at.a {
                    depth += 1;
                    match map.values().next() {
                        Some(Some(inner)) => {
                            at = inner;
                            depth += 1;
                        }
                        _ => break,
                    }
                }
                depth
            },
        )
    }),
    (
        "an untagged enum that holds itself in a list",
        |levels, entry| {
            typed::<Either>(
                levels,
                entry,
                mixed,
                |levels| if levels.is_multiple_of(2) { "" } else { "null" },
                depth_of_either,
            )
        },
    ),
    (
        "an untagged enum that holds itself in a box",
        |levels, entry| {
            // An untagged enum, a box and a struct for each level: the most stack a level of all.
            typed::<Either>(levels, entry, |_| "{\"a\":", |_| "null", depth_of_either)
        },
    ),
];

/// The levels of `either` along its first element or member.
fn depth_of_either(either: &Either) -> usize {
    let (mut at, mut depth) = (either, 1);
    loop {
        let inner = match at {
            Either::Many(list) => list.first(),
            Either::One { a } => a.as_deref(),
        };
        match inner {
            Some(inner) => {
                at = inner;
                depth += 1;
            }
            None => return depth,
        }
    }
}

/// Runs `child_case` of this binary for one case (a reader, a shape and a depth; or `typed`,
/// a target, an entry point and a depth) in a process of its own; how the reader ended, or
/// how the process did.
fn in_a_child(case: &str) -> Result<(), String> {
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "child_case",
            "--ignored",
            "--test-threads=1",
            "--nocapture",
        ])
        .env("JSON_STACK_CASE", case)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    if output.status.success() {
        return Ok(());
    }
    let said = String::from_utf8_lossy(&output.stderr);
    let why = said
        .lines()
        .find(|line| line.contains("stack overflow") || line.contains("JSON_STACK:"))
        .unwrap_or("no message");
    Err(format!("{} ({why})", output.status))
}

/// What a reader must give for a text nested `levels` deep, whose bracket past the limit is
/// at `past`.
fn expected(gives: Gives, levels: usize, past: usize) -> Read {
    match (gives, levels > MAX_JSON_DEPTH) {
        (Gives::Value | Gives::Helped, false) => Read::Value(levels),
        (Gives::Text, false) | (Gives::Helped, true) => Read::Text,
        (Gives::Value | Gives::Text, true) => Read::TooDeep(past),
    }
}

/// The child of [`in_a_child`]: runs the case on a thread with the stack it names. Without the
/// variable it is not a child and does nothing.
#[test]
#[ignore = "not a test of its own: the child process of in_a_child, which runs it with --ignored"]
fn child_case() {
    let Ok(case) = std::env::var("JSON_STACK_CASE") else {
        return;
    };
    let typed = case.starts_with("typed ");
    let mut parts = case
        .trim_start_matches("typed ")
        .split(' ')
        .map(|part| part.parse::<usize>().unwrap());
    let (first, second, levels) = (
        parts.next().unwrap(),
        parts.next().unwrap(),
        parts.next().unwrap(),
    );
    let (ended, expected) = std::thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(move || {
            if typed {
                let case = TARGETS[first].1(levels, second);
                let (_, past) = case.text;
                let expected = if levels > MAX_JSON_DEPTH {
                    Read::TooDeep(past)
                } else {
                    Read::Value(levels)
                };
                (case.ended, expected)
            } else {
                let (_, gives, read) = READERS[first];
                let (text, past) = SHAPES[second].1(levels);
                (read(&text), expected(gives, levels, past))
            }
        })
        .unwrap()
        .join()
        .unwrap();
    assert!(ended == expected, "JSON_STACK: {ended:?}, not {expected:?}");
}

/// Every reader, every way of nesting, at the limit, a level past it and far past it (a text
/// no reader may follow to its bottom before it is refused).
fn check(readers: std::ops::Range<usize>) {
    let mut failures = Vec::new();
    for reader in readers {
        for (shape, (nesting, _)) in SHAPES.iter().enumerate() {
            for levels in [MAX_JSON_DEPTH, MAX_JSON_DEPTH + 1, 1000, 100_000] {
                if let Err(ended) = in_a_child(&format!("{reader} {shape} {levels}")) {
                    failures.push(format!(
                        "{}: {nesting} nested {levels} deep: {ended}",
                        READERS[reader].0
                    ));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} cases did not end as their stack must let them:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn the_door_reads_a_value_nested_to_the_limit_on_a_megabyte_of_stack() {
    check(0..5);
}

#[test]
fn the_slice_readers_read_a_text_nested_to_the_limit_on_a_megabyte_of_stack() {
    check(5..8);
}

/// A typed target is facet's to read, a call of itself for each level of the type and of an
/// untyped value below it: the door runs it on a stack that holds the text.
#[test]
fn a_list_or_map_of_values_is_read_to_the_limit_on_a_megabyte_of_stack() {
    assert_eq!(READERS[8].0, TYPED);
    check(8..9);
}

/// The typed targets facet reads by a call of itself for each level of the text: a struct
/// with an untyped member, structs that hold themselves in a list, in a box, in a map, in an
/// enum; through every entry point, read and written again.
#[test]
fn a_typed_target_is_read_to_the_limit_on_a_megabyte_of_stack() {
    let mut failures = Vec::new();
    for (target, (name, _)) in TARGETS.iter().enumerate() {
        for (entry, through) in ENTRIES.iter().enumerate() {
            for levels in [
                MAX_JSON_DEPTH - 1,
                MAX_JSON_DEPTH,
                MAX_JSON_DEPTH + 1,
                MAX_JSON_DEPTH + 2,
                1000,
                100_000,
            ] {
                if let Err(ended) = in_a_child(&format!("typed {target} {entry} {levels}")) {
                    failures.push(format!(
                        "{name} through {through}, nested {levels} deep: {ended}"
                    ));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} typed reads did not end as a megabyte of stack must let them:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// The typed cases are what they say, on the test's own thread: each target reads its text
/// to the depth asked, through every entry point.
#[test]
fn the_typed_cases_nest_as_deep_as_they_say() {
    for (name, case) in TARGETS {
        for levels in [1, 2, 3, 6, 7, MAX_JSON_DEPTH] {
            let (text, _) = case(levels, 0).text;
            assert_eq!(check_depth(text.as_bytes(), levels), Ok(()), "{name}");
            assert!(check_depth(text.as_bytes(), levels - 1).is_err(), "{name}");
            for (entry, through) in ENTRIES.iter().enumerate() {
                let case = case(levels, entry);
                assert_eq!(case.ended, Read::Value(levels), "{name} through {through}");
            }
        }
    }
}

#[test]
fn the_scans_and_helpers_take_a_text_nested_to_the_limit_on_a_megabyte_of_stack() {
    check(9..READERS.len());
}

/// The cases are what they say: a reader that gives a value gives it whole, on the test's own
/// thread, and the shapes nest as deep as asked.
#[test]
fn the_cases_nest_as_deep_as_they_say() {
    for (nesting, shape) in SHAPES {
        for levels in [1, 2, 5, MAX_JSON_DEPTH] {
            let (text, _) = shape(levels);
            assert_eq!(check_depth(text.as_bytes(), levels), Ok(()), "{nesting}");
            assert!(
                check_depth(text.as_bytes(), levels - 1).is_err(),
                "{nesting}"
            );
        }
        let (text, past) = shape(MAX_JSON_DEPTH + 1);
        assert_eq!(
            scan(text.as_bytes(), true),
            Err(UnsafeText::TooDeep { offset: past }),
            "{nesting}"
        );
        let (text, past) = shape(6);
        for (reader, gives, read) in READERS {
            assert_eq!(
                read(&text),
                expected(*gives, 6, past),
                "{reader}: {nesting}"
            );
        }
    }
}

/// A chain `levels` structs long, built without a call for each.
fn chain(levels: usize) -> Chain {
    let mut chain = Chain { a: None };
    for _ in 1..levels {
        chain = Chain {
            a: Some(Box::new(chain)),
        };
    }
    chain
}

/// 7 in `levels` arrays, built without a call for each.
fn arrays(levels: usize) -> Value {
    let mut value = Value::from(7u64);
    for _ in 0..levels {
        let mut array = facet_value::VArray::new();
        array.push(value);
        value = array.into();
    }
    value
}

/// The writer refuses a value whose text no reader would read back, typed or untyped, on a
/// megabyte of stack, and leaves what it was given to write into as it was; a value at the
/// limit it writes.
#[test]
fn a_value_nested_past_the_limit_is_refused_by_the_writer() {
    std::thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(|| {
            for levels in [MAX_JSON_DEPTH + 1, MAX_JSON_DEPTH + 2, 1000] {
                let mut out = b"kept".to_vec();
                let refused = write_json(&chain(levels), &mut out, WriteStyle::default());
                assert!(
                    matches!(
                        refused,
                        Err(EncodeError::NotRepresentable {
                            what: Unrepresentable::TooDeep,
                            ..
                        })
                    ),
                    "a chain of {levels}: {refused:?}"
                );
                assert_eq!(out, b"kept");
                let refused = to_json(&arrays(levels));
                assert!(
                    matches!(
                        refused,
                        Err(EncodeError::NotRepresentable {
                            what: Unrepresentable::TooDeep,
                            ..
                        })
                    ),
                    "{levels} arrays: {refused:?}"
                );
            }
            let mut out = b"kept".to_vec();
            write_json(&chain(MAX_JSON_DEPTH), &mut out, WriteStyle::default()).unwrap();
            let written = std::str::from_utf8(&out).unwrap();
            let text = written.strip_prefix("kept").unwrap();
            assert_eq!(check_depth(text.as_bytes(), MAX_JSON_DEPTH), Ok(()));
            assert!(check_depth(text.as_bytes(), MAX_JSON_DEPTH - 1).is_err());
            let text = to_json(&arrays(MAX_JSON_DEPTH)).unwrap();
            assert_eq!(read_of(from_str_jsonc::<Value>(&text)), Read::Value(128));
        })
        .unwrap()
        .join()
        .unwrap();
}

/// A `Value` as facet-json spells it is written at the limit and refused past it, at the
/// array past the limit, on a megabyte of stack: facet-json's writer calls itself for each
/// level and took the megabyte at some 80.
#[test]
fn a_value_is_spelt_by_facet_json_to_the_limit_on_a_megabyte_of_stack() {
    std::thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(|| {
            for levels in [1, 64, MAX_JSON_DEPTH - 1, MAX_JSON_DEPTH] {
                let text = to_facet_json(&arrays(levels)).unwrap();
                assert_eq!(
                    text,
                    format!("{}7{}", "[".repeat(levels), "]".repeat(levels))
                );
            }
            let (text, _) = nest(MAX_JSON_DEPTH, mixed);
            let value = from_str_jsonc::<Value>(&text).unwrap();
            let spelt = to_facet_json(&value).unwrap();
            assert_eq!(spelt, text);
            for levels in [MAX_JSON_DEPTH + 1, MAX_JSON_DEPTH + 2, 1000] {
                match to_facet_json(&arrays(levels)) {
                    Err(EncodeError::NotRepresentable { at, what }) => {
                        assert_eq!(what, Unrepresentable::TooDeep, "{levels} arrays");
                        // The array that would be open inside 128 others.
                        assert_eq!(at.to_string(), vec!["0"; MAX_JSON_DEPTH].join("/"));
                    }
                    other => panic!("{levels} arrays: {other:?}"),
                }
            }
            // A scalar has no level, and a wide value is not a deep one.
            assert_eq!(to_facet_json(&Value::from(7u64)).unwrap(), "7");
            let wide = from_str_jsonc::<Value>(r#"{"a": [1, [2], {"b": []}], "c": {"d": 1.0}}"#);
            assert_eq!(
                to_facet_json(&wide.unwrap()).unwrap(),
                r#"{"a":[1,[2],{"b":[]}],"c":{"d":1}}"#
            );
        })
        .unwrap()
        .join()
        .unwrap();
}
