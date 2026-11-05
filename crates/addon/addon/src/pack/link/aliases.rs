//! The names of the game's own content that no definition of a stack and no generated list
//! has, used by every rule group through the lookups of [`LinkCx`], so that every group
//! answers the same for one name. The lists live in the vanilla data crate
//! (`bedrock_addon_core::vanilla::names`), each saying what it holds and where each name
//! comes from; the tests beside the lookups hold them against the game's files when those
//! are available.
//!
//! All of them stand with any host: the tables are the crate's own, and a host that answers
//! for a kind is not asked about these names (a host can add to them, never take one away).
//!
//! [`LinkCx`]: super::LinkCx

#![allow(
    unused_imports,
    reason = "the frame of the rule packages: which lists are used depends on the enabled \
              features"
)]

pub(crate) use crate::vanilla::names::*;
