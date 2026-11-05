//! `sounds/sound_definitions.json` of the resource packs: the sound events and the audio each
//! plays. The client merges every pack's copy as JSON before it reads the result, lowest pack
//! first: an object merges into an object member by member, and anything else (an array, a
//! number, `null`) replaces what the lower packs had. The layers of the [`Merged`] are the
//! packs' copies as written.
//!
//! The file has a version chain ([`history`]); the version steps run on each copy before the
//! merge.
//!
//! A finding about a merged sound definition is located in the file of the highest pack that
//! writes that definition, also when the member it is about was written by a lower pack and
//! left as it was above: the merge keeps one origin for each definition, not one for each
//! member.
//!
//! [`Merged`]: bedrock_addon_core::document::Merged

#![cfg_attr(docsrs, feature(doc_cfg))]

mod definitions;
pub mod history;
mod load;
mod upgrade;

pub use definitions::{
    CATEGORIES, ExternalEventParameter, SoundAction, SoundDefinition, SoundDefinitions, SoundEntry,
    SoundEntryForm,
};

#[doc(hidden)]
pub use load::LOADER;
