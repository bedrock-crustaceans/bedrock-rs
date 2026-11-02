//! The files of a resource stack as the texture rules see them: which pack has which path.
//! Every lookup of a texture path goes through here. Only names are held: no rule of the load
//! needs an image's bytes.

use indexmap::IndexMap;

use super::image::ImageFormat;
use bedrock_addon_core::load::StackCx;
use bedrock_addon_core::pack_identity::{PackPath, PackRef};

/// The directory the texture files live in.
const TEXTURES: &str = "textures/";

/// The extension of a texture set beside its texture.
pub const TEXTURE_SET: &str = ".texture_set.json";

/// One file of a pack.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexedFile {
    /// The file's path in its pack, subpack folder included.
    pub path: PackPath,
}

/// The files of one pack of the stack, by the path the stack reads them at (a subpack's file
/// stands at its key).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackFiles {
    /// The pack.
    pub pack: PackRef,
    /// The files by key, bytewise.
    pub files: IndexMap<String, IndexedFile>,
}

/// The files of every pack of a stack, top first.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StackFiles {
    /// One entry per pack of the stack, in stack order.
    pub packs: Vec<PackFiles>,
}

/// Where a lookup found a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Found<'a> {
    /// The pack's place in the stack; 0 is the top.
    pub stack_index: usize,
    /// The key the file was found at.
    pub key: &'a str,
    /// The file.
    pub file: &'a IndexedFile,
}

impl StackFiles {
    /// The highest pack that has `stem` with one of `extensions`; inside a pack the extensions
    /// are tried in order.
    // The stack is searched top-first, and inside each pack the extension list in order.
    #[must_use]
    pub fn first<'a>(&'a self, stem: &str, extensions: &[&str]) -> Option<Found<'a>> {
        self.packs
            .iter()
            .enumerate()
            .find_map(|(index, _)| self.in_pack(index, stem, extensions))
    }

    /// `stem` with one of `extensions` in the pack at `stack_index` only.
    #[must_use]
    pub fn in_pack<'a>(
        &'a self,
        stack_index: usize,
        stem: &str,
        extensions: &[&str],
    ) -> Option<Found<'a>> {
        let pack = self.packs.get(stack_index)?;
        extensions.iter().find_map(|extension| {
            let name = format!("{stem}{extension}");
            let (key, file) = pack.files.get_key_value(name.as_str())?;
            Some(Found {
                stack_index,
                key,
                file,
            })
        })
    }
}

/// The extensions an atlas layer's image is looked up with.
// The extensions the game loads an atlas layer's image with.
pub const ATLAS_IMAGE: [&str; 7] = [TEXTURE_SET, ".tga", ".png", ".jpg", ".jpeg", ".hdr", ".ktx"];

/// The extensions an image without a texture set is looked up with.
// The extensions the game loads an image with.
pub const IMAGE: [&str; 6] = [".tga", ".png", ".jpg", ".jpeg", ".hdr", ".ktx"];

/// The extensions a texture path is loaded with: the path as written first.
// The extensions the game loads a texture path with: the path as written first, then the
// image extensions.
pub const VERBATIM_OR_IMAGE: [&str; 7] = ["", ".tga", ".png", ".jpg", ".jpeg", ".hdr", ".ktx"];

/// Every file below `textures/`, and every image and texture set anywhere in the pack (a
/// texture path may point outside `textures/`).
#[must_use]
pub fn is_texture_file(path: &PackPath) -> bool {
    let path = path.as_str();
    path.starts_with(TEXTURES)
        || ImageFormat::from_path_ignoring_case(path).is_some()
        || path.ends_with("texture_set.json")
}

/// The texture files of each pack of the stack, by name: what the stack lists, bytes unread.
#[must_use]
pub fn listed_files(stack: &StackCx<'_>) -> StackFiles {
    let packs = stack.packs.iter().map(|entry| {
        let files = entry
            .files()
            .iter()
            .filter(|listed| is_texture_file(&listed.key))
            .map(|listed| {
                let path = listed.path.clone();
                (listed.key.as_str().to_owned(), IndexedFile { path })
            })
            .collect::<IndexMap<_, _>>();
        PackFiles {
            pack: entry.pack.clone(),
            files,
        }
    });
    StackFiles {
        packs: packs.collect(),
    }
}
