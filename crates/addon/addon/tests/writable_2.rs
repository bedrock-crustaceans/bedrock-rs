#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![allow(
    dead_code,
    unused_imports,
    reason = "the harness is the probe's of every document type"
)]
#![cfg(feature = "full")]
//! The probe of `writable.rs` for part of the twelve document types of the most texts (animation,
//! atmosphere, fog, music_definitions, particle): a test binary of its own, since a test binary
//! runs under a limit of CPU seconds. The rest of the twelve are probed by `writable_3.rs` and
//! `writable_4.rs`. The harness and its rules are `writable.rs`'s, whose
//! `the_binaries_of_the_probe_share_out_every_document_type` holds the four lists against each
//! other.

#[macro_use]
#[path = "common/document_types.rs"]
mod document_types;
#[path = "common/places.rs"]
mod places;

/// The document types this binary probes.
macro_rules! in_this_binary {
    (animation { $($body:tt)* }) => { $($body)* };
    (atmosphere { $($body:tt)* }) => { $($body)* };
    (fog { $($body:tt)* }) => { $($body)* };
    (music_definitions { $($body:tt)* }) => { $($body)* };
    (particle { $($body:tt)* }) => { $($body)* };
    ($other:ident $body:tt) => {};
}

include!("writable/harness.rs");
