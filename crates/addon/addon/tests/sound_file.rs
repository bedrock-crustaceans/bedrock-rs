//! The sound files family (`bedrock_addon::families::sound_file`, the audio files below
//! `sounds/` of the resource packs): the listing, the probe order, what is ignored, and what a
//! load asks the host for.

#![cfg(feature = "sound_file")]

#[path = "sound_file/assets.rs"]
mod assets;
#[path = "common/sound_packs.rs"]
mod common;
// What a load of the audio and of the three documents beside it asks the host for.
#[cfg(all(
    feature = "sound_definitions",
    feature = "sounds_json",
    feature = "music"
))]
#[path = "sound_file/requests.rs"]
mod requests;
#[cfg(all(
    feature = "sound_definitions",
    feature = "sounds_json",
    feature = "music"
))]
#[path = "common/responder.rs"]
mod responder;
