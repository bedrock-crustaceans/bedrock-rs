#![allow(
    dead_code,
    unused_imports,
    reason = "the harness is the probe's of every family"
)]
#![cfg(feature = "full")]
//! The legacy probe of `writable_legacy.rs` for the item, a family of many texts in a test
//! binary of its own (the harness and its rules are that file's).

#[allow(
    unused_macros,
    reason = "the list of document types is the other probes'"
)]
#[path = "common/document_types.rs"]
mod document_types;
#[path = "common/places.rs"]
mod places;

include!("writable_legacy/harness.rs");
include!("writable_legacy/families.rs");

use bedrock_addon::families::item::history;

legacy_families! {
    item: ["item"], ["minecraft:item"], [
        ("a pack", bedrock_addon::families::item::Item, "items/x.json", LoadContext::detached()),
        ("an allow-listed pack", bedrock_addon::families::item::Item, "items/x.json", allow_listed()),
        // The 1.20.80 list of custom components needs the Beta APIs below 1.21.10; its step's
        // documents loaded only as versionless legacy items, which 1.26.60 refuses.
        ("an allow-listed pack with the Beta APIs", bedrock_addon::families::item::Item, "items/x.json", {
            let mut cx = allow_listed();
            cx.beta_apis = true;
            cx
        }),
    ], steps![allow_listed;
        history::v1_10_0::Item,
        history::v1_16_100::Item,
        history::v1_19_83::Item,
        history::v1_20_0::Item,
        history::v1_20_20::Item,
        history::v1_20_30::Item,
        history::v1_20_40::Item,
        history::v1_20_50::Item,
        history::v1_20_60::Item,
        history::v1_20_80::Item,
        history::v1_21_10::Item,
        history::v1_21_30::Item,
        history::v1_21_40::Item,
        history::v1_21_50::Item,
        history::v1_21_60::Item,
        history::v1_21_80::Item,
        history::v1_21_90::Item,
        history::v1_21_110::Item,
        history::v1_26_0::Item,
        history::v1_26_30::Item,
        bedrock_addon::families::item::Item,
    ];
}
