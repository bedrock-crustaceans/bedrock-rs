//! The version-chain machinery struct selection per range model, the
//! envelope, a toy family through `version_chain!` and `#[derive(Components)]`, the generic
//! component checks, and `step_output`, which runs one step.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "versioning/toy.rs"]
mod toy;

#[path = "versioning/chain.rs"]
mod chain;
#[path = "versioning/close.rs"]
mod close;
#[path = "versioning/components.rs"]
mod components;
#[path = "versioning/envelope.rs"]
mod envelope;
#[path = "versioning/keep.rs"]
mod keep;
#[path = "versioning/legacy.rs"]
mod legacy;
#[path = "versioning/ranges.rs"]
mod ranges;
#[path = "versioning/reading.rs"]
mod reading;
#[path = "versioning/stages.rs"]
mod stages;
#[path = "versioning/step_output.rs"]
mod step_output;
