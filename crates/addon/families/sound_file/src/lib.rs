//! The audio files of the stack's `sounds/` trees: which file a sound entry's `name` resolves
//! to, and the lint rules over them. Vanilla opens audio only when a sound is first played;
//! nothing here changes what is loaded, and no audio file is read.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod files;
mod load;

pub use files::{AudioContainer, AudioExt, SOUND_FILE_KIND, SoundAsset, SoundFiles};

#[doc(hidden)]
pub use load::load_files;
