//! The vocabulary of the rule files: the context, the sink, the finding builders, the kinds
//! and targets of the lookups and the resolvers of the shared descriptor types. A rule file
//! starts with `use crate::pack::link::prelude::*;` and uses the part its rows need.

#![allow(
    unused_imports,
    reason = "unused_imports judges a re-export per feature set: with some families off no rule \
              file left compiled names part of this vocabulary, and the next feature set does"
)]

#[cfg(feature = "item")]
pub(crate) use super::item_descriptor;
pub(crate) use super::{
    At, Findings, LinkCx, Target, Validator, View, canonical, invalid, kinds, texture_not_found,
    unknown_sound, unresolved, unresolved_lang_key, validator_miss, validator_text,
};
#[cfg(any(feature = "block", feature = "item"))]
pub(crate) use super::{Member, States};
#[cfg(feature = "block")]
pub(crate) use super::{block_descriptor, block_specifier};
