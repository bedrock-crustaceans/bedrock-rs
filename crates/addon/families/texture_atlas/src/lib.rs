//! Texture atlases: `textures/terrain_texture.json` and `textures/item_texture.json`, the tiles
//! of the two atlases the client builds. Vanilla merges the copies of every pack as JSON,
//! highest priority first (`patchJson`), and reads the merged document once; the family's one
//! struct serves both files ([`AtlasKind`] tells them apart).
//!
//! The image paths of the merged atlas are looked up in the stack's file listing, so the load
//! can say which tile has no image and which variations the pack filter drops.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod atlas;
mod atlas_load;
mod atlas_read;
mod load;
mod patch;
mod texts;
mod write;

use bedrock_addon_core::diagnostic::Diagnostic;
use bedrock_addon_core::document::{Merged, RawLayer};
use bedrock_addon_support::texture::files::StackFiles;

pub use atlas::{
    AtlasItem, AtlasKind, AtlasTextures, Color, Face, Isotropic, MAX_WEIGHT, MIN_WEIGHT,
    ResolvedLayer, TextureAtlas, TextureElement, TextureLayer, Tint, TintMapColor, VariationEntry,
};

#[doc(hidden)]
pub use load::LOADER;

/// The atlases of a resource stack (`rp.texture_atlases()`).
#[derive(Debug)]
#[non_exhaustive]
pub struct TextureAtlases {
    /// The terrain atlas: every pack's `textures/terrain_texture.json`, merged as JSON and read
    /// once. When the stack builds no atlas ([`terrain`](Self::terrain) is `None`), the value
    /// is an atlas without a name and without tiles.
    pub terrain_atlas: Merged<TextureAtlas, RawLayer>,
    /// The item atlas: every pack's `textures/item_texture.json`, merged the same way.
    pub item_atlas: Merged<TextureAtlas, RawLayer>,
    /// Whether the stack builds each atlas, terrain first.
    built: [bool; 2],
    /// The files of the stack, which the images of the tiles are looked up in.
    files: StackFiles,
}

impl TextureAtlases {
    /// The atlases of a stack, with whether each was built.
    pub(crate) fn new(
        terrain_atlas: Merged<TextureAtlas, RawLayer>,
        item_atlas: Merged<TextureAtlas, RawLayer>,
        built: [bool; 2],
        files: StackFiles,
    ) -> Self {
        Self {
            terrain_atlas,
            item_atlas,
            built,
            files,
        }
    }

    /// The terrain atlas, when the stack builds one: at least one pack has the file, and the
    /// merged document names its `texture_name`.
    #[must_use]
    pub fn terrain(&self) -> Option<&TextureAtlas> {
        self.built[0].then(|| self.terrain_atlas.value())
    }

    /// The item atlas, when the stack builds one.
    #[must_use]
    pub fn item(&self) -> Option<&TextureAtlas> {
        self.built[1].then(|| self.item_atlas.value())
    }

    /// The layers vanilla keeps of the element at `index` of `item`, a tile of one of this
    /// stack's atlases, with their final weights.
    ///
    /// For an element with `variations`, each layer's image is looked up in the stack and only
    /// the layers at the highest pack that has one stay, their weights scaled to sum to 1. An
    /// image in no pack, and a layer without a path, counts as above every pack: one of them
    /// drops every variation that was found.
    #[must_use]
    pub fn effective_variations<'a>(
        &self,
        item: &'a AtlasItem,
        index: usize,
    ) -> Vec<ResolvedLayer<'a>> {
        let layers = item.layers(index);
        let mut kept: Vec<ResolvedLayer<'a>> = match item.elements().get(index) {
            Some(TextureElement::Variations { variations, .. }) => {
                let paths = atlas_load::variation_paths(variations);
                let keep = atlas_load::kept_variations(&paths, &self.files);
                layers
                    .into_iter()
                    .zip(keep)
                    .filter_map(|(layer, keep)| keep.then_some(layer))
                    .collect()
            }
            // `variations` present and no array: the one layer goes through the same filter.
            Some(TextureElement::Layer(layer))
                if layer.extra.get("variations").is_some_and(|v| !v.is_null()) =>
            {
                layers
            }
            _ => return layers,
        };
        let total: f32 = kept.iter().map(|layer| layer.weight).sum();
        for layer in &mut kept {
            layer.weight *= 1.0 / total;
        }
        kept
    }

    /// Every finding of the group: the terrain atlas, then the item atlas.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.terrain_atlas
            .diagnostics()
            .chain(self.item_atlas.diagnostics())
    }
}
