#![allow(
    dead_code,
    unused_imports,
    reason = "the harness is the probe's of every family"
)]
#![cfg(feature = "full")]
//! The legacy probe of `writable_legacy.rs` for the entity, second binary: the documents of
//! 1.13.0 up to 1.15.0 (see `writable_legacy_entity.rs`).

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
include!("writable_legacy/entity.rs");

entity_band!(entity_1_13_to_1_15, (1, 13, 0), (1, 15, 0));
