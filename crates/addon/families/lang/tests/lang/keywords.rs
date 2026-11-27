//! Which entries reach a table and under which key (schema.md rule 8): the prefix, allow and
//! block lists of vanilla's reader, and the pack keywords `pack.name` / `pack.description`.

use std::collections::BTreeMap;

use bedrock_addon_core::pack_identity::{PackIdVersion, PackType};
use bedrock_addon_core::version::SemVersion;
use bedrock_addon_family_lang::{
    KeyFilter, LangFile, RESERVED_KEYS, is_pack_keyword, pack_key_prefix,
};
use uuid::Uuid;

use super::common::*;

/// The identity README F.8 gives as its example: the vanilla resource pack at 0.0.1.
fn vanilla_pack() -> PackIdVersion {
    PackIdVersion::new(
        Uuid::parse_str("ee649bcf-256c-4013-9068-6a802b89d756").unwrap(),
        SemVersion::new(0, 0, 1),
        PackType::Resource,
    )
}

/// The table `filter` selects from a file with these bytes.
fn select(bytes: &[u8], filter: &KeyFilter<'_>) -> BTreeMap<String, String> {
    LangFile::from_bytes(en_us(), bytes)
        .0
        .select(filter)
        .strings
}

fn map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

#[test]
fn ut_prefix_stores_every_key_under_the_prefix() {
    let read = lang("ut_prefix");
    assert!(!read.table.strings.contains_key("key.1"));
    assert_eq!(read.table.strings.len(), 4);
}

#[test]
fn ut_allowed_registers_only_the_allowed_keys() {
    let read = lang("ut_allowed");
    assert_eq!(read.table.strings.len(), 2);
}

#[test]
fn ut_blocked_registers_every_key_but_the_blocked_ones() {
    let read = lang("ut_blocked");
    assert_eq!(read.table.strings.len(), 2);
}

#[test]
fn a_blocked_key_is_skipped_even_when_it_is_allowed() {
    let filter = KeyFilter {
        exclude: &["a"],
        include: &["a", "b"],
        prefix: "",
    };
    assert_eq!(select(b"a=1\nb=2\nc=3\n", &filter), map(&[("b", "2")]));
}

#[test]
fn the_lists_match_the_key_without_the_prefix() {
    let with_key = KeyFilter {
        exclude: &[],
        include: &["key.2"],
        prefix: "pre.",
    };
    let with_prefixed_key = KeyFilter {
        include: &["pre.key.2"],
        ..with_key
    };
    let blocked = KeyFilter {
        exclude: &["pre.key.2"],
        include: &[],
        prefix: "pre.",
    };
    let bytes = b"key.1=a\nkey.2=b\n";
    assert_eq!(select(bytes, &with_key), map(&[("pre.key.2", "b")]));
    assert_eq!(select(bytes, &with_prefixed_key), map(&[]));
    assert_eq!(select(bytes, &blocked).len(), 2);
}

#[test]
fn the_lists_match_the_lower_cased_key_and_the_prefix_is_stored_as_given() {
    let filter = KeyFilter {
        exclude: &[],
        include: &["key.one"],
        prefix: "PRE.",
    };
    assert_eq!(
        select(b"Key.One=a\nKEY.TWO=b\n", &filter),
        map(&[("PRE.key.one", "a")])
    );
    // a list entry that is not lower-case matches nothing
    let upper = KeyFilter {
        exclude: &[],
        include: &["Key.One"],
        prefix: "",
    };
    assert_eq!(select(b"Key.One=a\n", &upper), map(&[]));
}

#[test]
fn reserved_keys_never_reach_the_global_table() {
    let read = lang("reserved_keys");
    // the stack loader's filter is exactly the one the expectation spells out
    assert_eq!(read.file.translations().strings, read.table.strings);
    assert_eq!(read.file.translations().strings, map(&[("other", "Z")]));
    assert_eq!(KeyFilter::GLOBAL.exclude, RESERVED_KEYS);
    assert!(KeyFilter::GLOBAL.include.is_empty());
    assert_eq!(KeyFilter::GLOBAL.prefix, "");
}

#[test]
fn reserved_keys_are_the_pack_keywords_under_the_uuid_and_version() {
    let read = lang("reserved_keys");
    let keywords = read.file.pack_keywords(&vanilla_pack());
    assert_eq!(
        keywords.strings,
        map(&[
            (
                "ee649bcf-256c-4013-9068-6a802b89d7560.0.1pack.description",
                "Y"
            ),
            ("ee649bcf-256c-4013-9068-6a802b89d7560.0.1pack.name", "X"),
        ])
    );
    assert_eq!(
        keywords.pack_keyword(&vanilla_pack(), "pack.name"),
        Some("X")
    );
    assert_eq!(
        keywords.pack_keyword(&vanilla_pack(), "pack.description"),
        Some("Y")
    );
    // another pack's keywords are not this pack's, and a global key is no keyword
    let other = PackIdVersion::new(
        Uuid::from_u128(7),
        SemVersion::new(0, 0, 1),
        PackType::Resource,
    );
    assert_eq!(keywords.pack_keyword(&other, "pack.name"), None);
    assert_eq!(keywords.pack_keyword(&vanilla_pack(), "other"), None);
}

#[test]
fn the_pack_key_prefix_is_the_uuid_directly_followed_by_the_version() {
    assert_eq!(
        pack_key_prefix(&vanilla_pack()),
        "ee649bcf-256c-4013-9068-6a802b89d7560.0.1"
    );
    let filter = KeyFilter::pack_keywords("p");
    assert_eq!(filter.include, RESERVED_KEYS);
    assert!(filter.exclude.is_empty());
    assert_eq!(filter.prefix, "p");
}

#[test]
fn a_pack_keyword_is_exactly_one_of_the_two_reserved_keys() {
    assert!(is_pack_keyword("pack.name"));
    assert!(is_pack_keyword("pack.description"));
    for text in [
        "Pack.Name",
        "pack.name ",
        " pack.name",
        "pack.names",
        "pack",
        "",
    ] {
        assert!(!is_pack_keyword(text), "{text:?}");
    }
}
