//! The document types of this crate that hold themselves and are read by facet, nested to
//! `MAX_JSON_DEPTH` and a level past it, on a thread with a megabyte of stack: a script
//! type descriptor (`RawTypeDesc`, itself in a box: the most stack a level of any type here)
//! and a block's legacy event (`DeprecatedDefinitionEvent`, itself in a list).
//!
//! facet reads them by a call of itself for each level: 55 KiB a level for the first in a
//! debug build, which overflowed a megabyte at 19 levels. The door gives the read its stack.
//! Its own binary: an overflow aborts the process.

#![cfg(all(feature = "script", feature = "block"))]

use bedrock_addon::families::block::component::DeprecatedDefinitionEvent;
use bedrock_addon::families::script::meta::TypeDescriptor;
use bedrock_addon::families::script::meta::model::script::RawTypeDesc;
use bedrock_addon::json::{JsonSlice, MAX_JSON_DEPTH, check_depth, parse_at, to_json};

/// The stack of the reading thread: half the default of a spawned thread.
const SMALL_STACK: usize = 1024 * 1024;

/// `run` on a thread with a megabyte of stack.
fn on_a_small_stack<T: Send + 'static>(run: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(run)
        .unwrap()
        .join()
        .expect("the read does not panic")
}

/// An `optional` of an `optional` …, `levels` objects deep.
fn descriptor(levels: usize) -> String {
    let open =
        r#"{"name": "optional", "is_bind_type": false, "is_errorable": false, "optional_type": "#;
    let bottom = r#"{"name": "int32", "is_bind_type": false, "is_errorable": false}"#;
    format!(
        "{}{bottom}{}",
        open.repeat(levels - 1),
        "}".repeat(levels - 1)
    )
}

/// A `sequence` in a `sequence` …, `levels` arrays and objects deep.
fn event(levels: usize) -> String {
    let turns = (levels - 1) / 2;
    // An object closes the nesting; an odd level more is an empty list in it.
    let bottom = if levels.is_multiple_of(2) {
        r#"{"sequence": []}"#
    } else {
        "{}"
    };
    format!(
        "{}{bottom}{}",
        r#"{"sequence": ["#.repeat(turns),
        "]}".repeat(turns)
    )
}

#[test]
fn the_texts_nest_as_deep_as_asked() {
    for levels in [1, 2, 3, MAX_JSON_DEPTH, MAX_JSON_DEPTH + 1] {
        for text in [descriptor(levels), event(levels)] {
            assert_eq!(check_depth(text.as_bytes(), levels), Ok(()), "{text}");
            assert!(check_depth(text.as_bytes(), levels - 1).is_err(), "{text}");
        }
    }
}

#[test]
fn a_type_descriptor_nested_to_the_limit_is_read_lowered_and_written() {
    for levels in [MAX_JSON_DEPTH - 1, MAX_JSON_DEPTH] {
        let depth = on_a_small_stack(move || {
            let text = descriptor(levels);
            let raw: RawTypeDesc = parse_at(JsonSlice::new(&text)).expect("within the limit");
            TypeDescriptor::lower(&raw).expect("a descriptor of optionals");
            let written = to_json(&raw).expect("every member has a spelling");
            let again: RawTypeDesc = parse_at(JsonSlice::new(&written)).expect("what was written");
            assert!(again == raw, "the descriptor written reads as another");
            let (mut at, mut depth) = (&raw, 1);
            while let Some(inner) = &at.optional_type {
                at = inner;
                depth += 1;
            }
            depth
        });
        assert_eq!(depth, levels);
    }
}

#[test]
fn a_legacy_event_nested_to_the_limit_is_read_and_written() {
    for levels in [MAX_JSON_DEPTH - 1, MAX_JSON_DEPTH] {
        let depth = on_a_small_stack(move || {
            let text = event(levels);
            let read: DeprecatedDefinitionEvent =
                parse_at(JsonSlice::new(&text)).expect("within the limit");
            let written = to_json(&read).expect("every member has a spelling");
            let again: DeprecatedDefinitionEvent =
                parse_at(JsonSlice::new(&written)).expect("what was written");
            assert!(again == read, "the event written reads as another");
            let (mut at, mut depth) = (&read, 1);
            while let Some(inner) = at.sequence.first() {
                at = inner;
                depth += 2;
            }
            depth
        });
        // An empty list at the bottom is a level the value does not show.
        assert_eq!(depth + usize::from(levels.is_multiple_of(2)), levels);
    }
}

#[test]
fn a_level_past_the_limit_is_an_error_for_both() {
    for levels in [MAX_JSON_DEPTH + 1, 1000, 100_000] {
        let refused = on_a_small_stack(move || {
            let (descriptor, event) = (descriptor(levels), event(levels));
            let descriptor = parse_at::<RawTypeDesc>(JsonSlice::new(&descriptor)).err();
            let event = parse_at::<DeprecatedDefinitionEvent>(JsonSlice::new(&event)).err();
            [descriptor, event].map(|error| error.map(|error| error.error.to_string()))
        });
        for error in refused {
            let error = error.expect("a text past the limit is refused");
            assert!(error.contains("at most 128 levels"), "{error}");
        }
    }
}
