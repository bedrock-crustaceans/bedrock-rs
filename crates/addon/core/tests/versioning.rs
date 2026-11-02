#![allow(clippy::pedantic, missing_docs)]
//! The version-chain machinery struct selection per range model, the
//! envelope, a toy family through `version_chain!` and `#[derive(Components)]`, the generic
//! component checks, and `step_output`, which runs one step.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "versioning/toy.rs"]
mod toy;

#[path = "versioning/chain.rs"]
mod chain;
#[path = "versioning/components.rs"]
mod components;
#[path = "versioning/envelope.rs"]
mod envelope;
#[path = "versioning/ranges.rs"]
mod ranges;
#[path = "versioning/stages.rs"]
mod stages;
#[path = "versioning/step_output.rs"]
mod step_output;
