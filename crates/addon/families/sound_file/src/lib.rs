//! The audio files of the stack's `sounds/` trees: which file a sound entry's `name` resolves
//! to, and the lint rules over them. Vanilla opens audio only when a sound is first played;
//! nothing here changes what is loaded, and no audio file is read.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod files;

use bedrock_addon_core::diagnostic::Diagnostic;
use bedrock_addon_core::load::Reported;
use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, Walk, WalkFile};

use files::index;

pub use files::{AudioContainer, AudioExt, SOUND_FILE_KIND, SoundAsset, SoundFiles};

/// Nothing to parse: the walk reads no file.
fn parse(_: &WalkFile, _: &LoadContext) {}

/// The index of the packs' listings.
fn combine(_: Vec<()>, cx: &StackCx<'_>) -> SoundFiles {
    index(cx.packs)
}

/// The loader of `rp.sound_files()`: the audio under `sounds/` of every resource pack, from
/// the packs' listings.
#[doc(hidden)]
pub const LOADER: CollectionLoader<SoundFiles, ()> = CollectionLoader {
    walk: Walk::none(),
    parse,
    combine,
    needs: &[],
    then: None,
};

impl Reported for SoundFiles {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.findings())
    }
}
