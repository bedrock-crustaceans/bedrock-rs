//! Loading the texture files of a resource stack: no file is walked, the packs' file listings
//! are all the index is built from.

use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, Walk, WalkFile};

use super::TextureFiles;
use bedrock_addon_core::diagnostic::Diagnostic;
use bedrock_addon_core::load::Reported;
use bedrock_addon_support::texture::files::listed_files;

/// Nothing to parse: the walk reads no file.
fn parse(_: &WalkFile, _: &LoadContext) {}

/// The index of the packs' listings.
fn combine(_: Vec<()>, cx: &StackCx<'_>) -> TextureFiles {
    TextureFiles::from_files(listed_files(cx))
}

/// The loader of `rp.texture_files()`: every texture file of every resource pack, from the
/// packs' listings.
pub const fn load_files() -> CollectionLoader<TextureFiles, ()> {
    CollectionLoader {
        walk: Walk::none(),
        parse,
        combine,
        needs: &[],
        then: None,
    }
}

impl Reported for TextureFiles {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(std::iter::empty())
    }
}
