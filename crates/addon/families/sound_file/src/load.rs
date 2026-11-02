//! Loading the audio files of a resource stack: no file is walked, the packs' file listings are
//! all the index is built from.

use bedrock_addon_core::load::{
    AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile, WalkOrder,
};
use bedrock_addon_core::pack_identity::PackPath;

use super::files::{SoundFiles, index};
use bedrock_addon_core::diagnostic::Diagnostic;
use bedrock_addon_core::load::Reported;

/// No file is read.
fn accepts(_: &PackPath) -> bool {
    false
}

/// Nothing to parse: the walk reads no file.
fn parse(_: &WalkFile, _: &LoadContext) {}

/// The index of the packs' listings.
fn combine(_: Vec<()>, cx: &StackCx<'_>) -> SoundFiles {
    index(cx.packs)
}

/// The loader of `rp.sound_files()`: the audio under `sounds/` of every resource pack, from
/// the packs' listings.
#[must_use]
pub const fn load_files() -> CollectionLoader<SoundFiles, ()> {
    CollectionLoader {
        walk: Walk {
            directories: &[],
            depth: ListDepth::Children,
            accept: accepts,
            order: WalkOrder::BottomUp,
            directory_major: false,
            app_data: &[],
            app_data_place: AppDataPlace::After,
            in_pack: None,
            shadowed: None,
            read_error: None,
        },
        parse,
        combine,
        needs: &[],
        then: None,
    }
}

impl Reported for SoundFiles {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.findings())
    }
}
