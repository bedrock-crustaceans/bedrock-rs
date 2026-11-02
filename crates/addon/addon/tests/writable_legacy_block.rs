#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![allow(
    dead_code,
    unused_imports,
    reason = "the harness is the probe's of every family"
)]
#![cfg(feature = "full")]
//! The legacy probe of `writable_legacy.rs` for the block, a family of many texts in a test
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

use bedrock_addon::families::block::history;

legacy_families! {
    block: ["block"], ["minecraft:block"], [
        ("a pack", bedrock_addon::families::block::Block, "blocks/x.json", LoadContext::detached()),
        ("an allow-listed pack", bedrock_addon::families::block::Block, "blocks/x.json", allow_listed()),
    ], steps![allow_listed;
        history::v1_16_0::Block,
        history::v1_16_100::Block,
        history::v1_18_0::Block,
        history::v1_19_10::Block,
        history::v1_19_20::Block,
        history::v1_19_30::Block,
        history::v1_19_40::Block,
        history::v1_19_80::Block,
        history::v1_20_10::Block,
        history::v1_20_60::Block,
        history::v1_21_30::Block,
        history::v1_21_60::Block,
        history::v1_21_90::Block,
        history::v1_26_0::Block,
    ];
}
