#![allow(clippy::pedantic, missing_docs)]
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![allow(
    dead_code,
    unused_imports,
    reason = "the harness is the probe's of every family"
)]
#![cfg(feature = "full")]
//! The legacy probe of `writable_legacy.rs` for the entity, sixth binary: the documents
//! from 1.26.30 below 1.26.40 (see `writable_legacy_entity.rs`; split by version).

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

entity_band!(entity_1_26_30_to_1_26_40, (1, 26, 30), (1, 26, 40));
