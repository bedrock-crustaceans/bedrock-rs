//! The texture files of a resource stack: the images and everything else below `textures/`, as
//! the texture rules see them. Only names are held: no rule of the load needs an image's bytes,
//! and the stack lists the packs' files, so nothing is read. The family answers which pack has
//! which path and which file the client loads for a texture path ([`TextureFiles::source`]).
//!
//! Nothing is read: the family holds names from the stack's file listing.

#![cfg_attr(docsrs, feature(doc_cfg))]

use bedrock_addon_core::pack_identity::{PackPath, PackRef};

use bedrock_addon_shared_shapes::texture::files::{StackFiles, TEXTURE_SET, VERBATIM_OR_IMAGE};

use bedrock_addon_core::diagnostic::Diagnostic;
use bedrock_addon_core::load::Reported;
use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_shared_shapes::texture::files::listed_files;
pub use bedrock_addon_shared_shapes::texture::image::{ImageFormat, ImageInfo};

/// The file the client loads for a texture path.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TextureSource {
    /// A texture set beside the texture.
    TextureSet {
        /// The pack's place in the stack.
        stack_index: usize,
    },
    /// An image.
    Image {
        /// The pack's place in the stack.
        stack_index: usize,
        /// The image's path in the pack.
        file: String,
    },
}

/// An image file of a pack of the stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TextureImage<'a> {
    /// The path the stack reads the file at (a subpack's file stands at its pack's path).
    pub key: &'a str,
    /// The file's path in its pack, subpack folder included: what a reader asks the pack for.
    pub path: &'a PackPath,
    /// The format by the file's extension; `None` for a file loaded by its exact name that
    /// has no image extension.
    pub format: Option<ImageFormat>,
}

/// The texture files of a resource stack (`rp.texture_files()`): which pack has which path.
///
/// Hand-written: it reads the packs' listings and no file's bytes (its walk is `Walk::none()`).
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct TextureFiles {
    /// The files of the stack.
    files: StackFiles,
}

impl TextureFiles {
    /// The files of the stack by pack, for the families that resolve against them.
    pub(crate) fn from_files(files: StackFiles) -> Self {
        Self { files }
    }

    /// The packs of the stack that hold a file at `path` (the path the stack reads it at: a
    /// subpack's file stands at its pack's path), top pack first. Only texture files are
    /// known: everything below `textures/`, and images and texture sets anywhere.
    pub fn packs_with<'a>(&'a self, path: &'a str) -> impl Iterator<Item = &'a PackRef> + 'a {
        self.files
            .packs
            .iter()
            .filter(move |pack| pack.files.contains_key(path))
            .map(|pack| &pack.pack)
    }

    /// The pack at `stack_index` of the stack, if it has one.
    #[allow(
        dead_code,
        reason = "used by the stack's composition and the link rules, which need other features"
    )]
    #[doc(hidden)]
    pub fn pack(&self, stack_index: usize) -> Option<&PackRef> {
        self.files.packs.get(stack_index).map(|pack| &pack.pack)
    }

    /// The files of the stack, for the link rules.
    #[allow(
        dead_code,
        reason = "used by the stack's composition and the link rules, which need other features"
    )]
    #[doc(hidden)]
    pub fn files(&self) -> &StackFiles {
        &self.files
    }

    /// The file the client loads for the texture path `location`: the highest pack with a
    /// texture set beside the path competes with the highest pack that has the path itself or
    /// the path with an image extension; the higher pack wins, and the texture set wins a
    /// tie. `None` when no pack has either (the client shows the missing texture).
    #[must_use]
    pub fn source(&self, location: &str) -> Option<TextureSource> {
        let set = self.files.first(location, &[TEXTURE_SET]);
        let image = self.files.first(location, &VERBATIM_OR_IMAGE);
        match (set, image) {
            (Some(set), Some(image)) if set.stack_index > image.stack_index => {
                Some(image_source(&image))
            }
            (Some(set), _) => Some(TextureSource::TextureSet {
                stack_index: set.stack_index,
            }),
            (None, Some(image)) => Some(image_source(&image)),
            (None, None) => None,
        }
    }

    /// The image at `stem` of the pack at `stack_index`, by the lookup of a texture path.
    #[allow(
        dead_code,
        reason = "used by the stack's composition and the link rules, which need other features"
    )]
    #[doc(hidden)]
    pub fn image_in<'a>(&'a self, stack_index: usize, stem: &str) -> Option<TextureImage<'a>> {
        let found = self.files.in_pack(stack_index, stem, &VERBATIM_OR_IMAGE)?;
        Some(TextureImage {
            key: found.key,
            path: &found.file.path,
            format: ImageFormat::from_path(found.key),
        })
    }
}

/// An image hit as a source.
fn image_source(image: &bedrock_addon_shared_shapes::texture::files::Found<'_>) -> TextureSource {
    TextureSource::Image {
        stack_index: image.stack_index,
        file: image.key.to_owned(),
    }
}

/// Nothing to parse: the walk reads no file.
fn parse(_: &WalkFile, _: &LoadContext) {}

/// The index of the packs' listings.
fn combine(_: Vec<()>, cx: &StackCx<'_>) -> TextureFiles {
    TextureFiles::from_files(listed_files(cx))
}

/// The loader of `rp.texture_files()`: every texture file of every resource pack, from the
/// packs' listings.
#[doc(hidden)]
pub const LOADER: CollectionLoader<TextureFiles, ()> = CollectionLoader {
    walk: Walk::none(),
    parse,
    combine,
    needs: &[],
    then: None,
};

impl Reported for TextureFiles {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(std::iter::empty())
    }
}
