#![allow(
    dead_code,
    unused_imports,
    reason = "the harness is the probe's of every family"
)]
#![cfg(feature = "full")]
//! The legacy probe of `writable_legacy.rs` for the entity, seventh binary: the documents
//! from 1.26.50 (see `writable_legacy_entity.rs`; split by version).

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

entity_band!(entity_from_1_26_50, (1, 26, 50), (99, 0, 0));
