//! `id::syntax::{split_namespace, has_namespace}` against the copies they stand for.

use bedrock_addon_core::id::syntax::{has_namespace, split_namespace};

/// The body of the three exact copies: ddui's `namespaced` (`write.rs` and `read.rs`) and
/// item's `has_namespace` (`history/document.rs`).
fn copied(text: &str) -> bool {
    text.split_once(':')
        .is_some_and(|(namespace, name)| !namespace.is_empty() && !name.is_empty())
}

const TEXTS: &[&str] = &[
    "minecraft:stone",
    "demo:a:b",
    "a::",
    "::a",
    ":a",
    "a:",
    ":",
    "",
    "stone",
    "a:b:",
    " :b",
    "MINECRAFT:Stone",
];

#[test]
fn has_namespace_is_the_three_copies() {
    for text in TEXTS {
        assert_eq!(has_namespace(text), copied(text), "{text:?}");
    }
    assert!(has_namespace("minecraft:stone"));
    // The namespace ends at the first `:`, so a second one is the name's.
    assert!(has_namespace("demo:a:b"));
    assert!(has_namespace("a::"));
    assert!(!has_namespace("::a"));
    assert!(!has_namespace("a:"));
    assert!(!has_namespace("stone"));
}

#[test]
fn split_namespace_splits_at_the_first_colon() {
    assert_eq!(
        split_namespace("minecraft:stone"),
        Some(("minecraft", "stone"))
    );
    assert_eq!(split_namespace("demo:a:b"), Some(("demo", "a:b")));
    assert_eq!(split_namespace(":a"), Some(("", "a")));
    assert_eq!(split_namespace("a:"), Some(("a", "")));
    assert_eq!(split_namespace("stone"), None);
    for text in TEXTS {
        assert_eq!(split_namespace(text), text.split_once(':'), "{text:?}");
    }
}
