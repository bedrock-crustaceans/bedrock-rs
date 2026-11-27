#![allow(
    dead_code,
    unused_imports,
    reason = "the harness is the probe's of every family"
)]
#![cfg(feature = "full")]
//! The legacy probe of `writable_legacy.rs` for the entity, a family of many texts in seven
//! test binaries of its own (the harness and its rules are that file's; a test binary runs
//! under a limit of CPU seconds): the documents below 1.13.0 here, those of 1.13.0 in
//! `writable_legacy_entity_2.rs`, of 1.15.0 up to 1.20.0 in `writable_legacy_entity_3.rs`,
//! of 1.20.0 up to 1.26.0 in `writable_legacy_entity_4.rs`, of 1.26.0 up to 1.26.30 in
//! `writable_legacy_entity_5.rs`, of 1.26.30 up to 1.26.50 in `writable_legacy_entity_6.rs`,
//! from 1.26.50 in `writable_legacy_entity_7.rs`.

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

entity_band!(entity_to_1_13, (0, 0, 0), (1, 13, 0));
