//! Loading the atlases of a resource stack: one walk over the two atlas files, one parse job
//! per file, and one combine that merges each atlas over the stack and looks the image paths of
//! the result up in the stack's file listing. No image is read.

use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, WalkFile};
use bedrock_addon_core::pack_identity::PackPath;

use super::TextureAtlases;
use super::atlas::AtlasKind;
use super::atlas_load::{AtlasCopy, merge_atlas, parse_copy};
use bedrock_addon_core::diagnostic::Diagnostic;
use bedrock_addon_core::load::Reported;
use bedrock_addon_support::texture::files::listed_files;
use bedrock_addon_support::texture::walk::walk;

/// One walked atlas file: which atlas it is a copy of, and the copy.
#[derive(Debug)]
pub struct AtlasFileParsed {
    /// The atlas.
    kind: AtlasKind,
    /// The copy.
    copy: Box<AtlasCopy>,
}

/// The files the walk reads: the two atlases.
fn accepts(path: &PackPath) -> bool {
    [AtlasKind::Terrain.path(), AtlasKind::Item.path()].contains(&path.as_str())
}

/// One file of the walk.
fn parse_walk_file(file: &WalkFile, _: &LoadContext) -> AtlasFileParsed {
    let kind = if file.key.as_str() == AtlasKind::Item.path() {
        AtlasKind::Item
    } else {
        AtlasKind::Terrain
    };
    AtlasFileParsed {
        kind,
        copy: Box::new(parse_copy(file)),
    }
}

/// The atlases, from every parsed file in walk order.
fn combine(parsed: Vec<AtlasFileParsed>, stack: &StackCx<'_>) -> TextureAtlases {
    let files = listed_files(stack);
    let (mut terrain, mut item) = (Vec::new(), Vec::new());
    for file in parsed {
        match file.kind {
            AtlasKind::Terrain => terrain.push(*file.copy),
            AtlasKind::Item => item.push(*file.copy),
        }
    }
    let (terrain_atlas, terrain_built) = merge_atlas(AtlasKind::Terrain, terrain, &files);
    let (item_atlas, item_built) = merge_atlas(AtlasKind::Item, item, &files);
    TextureAtlases::new(
        terrain_atlas,
        item_atlas,
        [terrain_built, item_built],
        files,
    )
}

/// The loader of `rp.texture_atlases()`: the terrain and item atlases of every resource pack,
/// each merged over the stack.
pub const LOADER: CollectionLoader<TextureAtlases, AtlasFileParsed> = CollectionLoader {
    walk: walk(accepts, None),
    parse: parse_walk_file,
    combine,
    needs: &[],
    then: None,
};

impl Reported for TextureAtlases {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}
