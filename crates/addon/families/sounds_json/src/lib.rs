//! `sounds.json` of the resource packs: which sound event each game event plays. The client
//! merges every pack's copy as JSON before it reads the result, lowest pack first: an object
//! merges into an object member by member, and anything else (an array, a number, `null`)
//! replaces what the lower packs had. The layers of the [`Merged`] are the packs' copies as
//! written.
//!
//! [`Merged`]: bedrock_addon_core::document::Merged

#![cfg_attr(docsrs, feature(doc_cfg))]

mod bindings;
mod events;
mod load;

pub use bindings::{
    BlockSounds, EntitySoundEntry, EntitySounds, EventSounds, InteractiveEntityEntry,
    InteractiveEntitySounds, InteractiveEventSounds, InteractiveSounds, NamedSounds, SoundBindings,
    SoundRange, SoundRef, SoundVariants,
};

#[doc(hidden)]
pub use events::is_level_sound_event;
#[doc(hidden)]
pub use load::load_bindings;
