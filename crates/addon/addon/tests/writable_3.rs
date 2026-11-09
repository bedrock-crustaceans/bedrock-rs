#![allow(
    dead_code,
    unused_imports,
    reason = "the harness is the probe's of every document type"
)]
#![cfg(feature = "full")]
//! The probe of `writable.rs` for part of the twelve document types of the most texts (entity,
//! feature, item, `ui_global_variables`): a test binary of its own, since a test binary runs under a
//! limit of CPU seconds. The rest of the twelve are probed by `writable_2.rs` and `writable_4.rs`.
//! The harness and its rules are `writable.rs`'s, whose
//! `the_binaries_of_the_probe_share_out_every_document_type` holds the four lists against each
//! other.

#[macro_use]
#[path = "common/document_types.rs"]
mod document_types;
#[path = "common/places.rs"]
mod places;

/// The document types this binary probes.
macro_rules! in_this_binary {
    (entity { $($body:tt)* }) => { $($body)* };
    (feature { $($body:tt)* }) => { $($body)* };
    (item { $($body:tt)* }) => { $($body)* };
    (ui_global_variables { $($body:tt)* }) => { $($body)* };
    ($other:ident $body:tt) => {};
}

include!("writable/harness.rs");
