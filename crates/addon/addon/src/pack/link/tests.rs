//! The frame of the link pass, tested with rules made up here, so that the frame's behaviour
//! does not depend on what the six rule groups report.

#[path = "tests_support.rs"]
pub(in crate::pack::link) mod support;

#[path = "tests_entry.rs"]
mod entry;
#[path = "tests_findings.rs"]
mod findings;
#[path = "tests_lookups.rs"]
mod lookups;
#[cfg(any(feature = "block", feature = "item"))]
#[path = "tests_shared.rs"]
mod shared;
#[path = "tests_wiring.rs"]
mod wiring;
