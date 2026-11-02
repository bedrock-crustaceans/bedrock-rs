//! What the stack makes of the texture families together: which file the client loads for a
//! texture path (the texture sets beside the images, resolved against the packs' files), and
//! the image of every layer of every loaded texture set.

use bedrock_addon_core::pack_identity::PackRef;

use super::{PackStack, Resource};
use crate::diagnostic::{Diagnostic, DiagnosticCode, Location, Phase, Severity};
use crate::document::{Document, Rejected};
use crate::families::texture_file::{ImageInfo, TextureFiles, TextureImage, TextureSource};
use crate::families::texture_set::{LayerKind, TextureSet, TextureSets, layer_path, payload_at};
use crate::id::TexturePath;

/// What the client loads for a texture path ([`PackStack::resolve_texture`]).
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct TextureFile<'a> {
    /// The pack whose file wins: the highest pack with a texture set beside the path or with
    /// an image at it; the texture set wins a tie.
    pub pack: &'a PackRef,
    /// That pack's place in the stack; 0 is the top.
    pub stack_index: usize,
    /// The texture set, when one wins and loads. `None` for a plain image (the client makes a
    /// set of one `color` layer from it) and for a set that did not load.
    pub set: Option<&'a Document<TextureSet>>,
    /// The texture set that won and did not load: the client shows the missing texture, and
    /// the image beside the set is not used instead.
    pub rejected: Option<&'a Rejected>,
    /// The image of the colour: the file itself for a plain image, the `color` layer's image
    /// (in the set's own pack) for a texture set. `None` for a uniform colour and for a set
    /// that did not load.
    pub image: Option<TextureImage<'a>>,
}

impl TextureFile<'_> {
    /// Whether the client shows the missing texture: the winning texture set did not load.
    #[must_use]
    pub fn is_missing(&self) -> bool {
        self.rejected.is_some()
    }
}

/// The image of one layer of a loaded texture set: the file a caller reads to have the
/// image's own rules checked ([`check`](Self::check)). Loading a stack reads no image.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct LayerImage<'a> {
    /// The texture set.
    pub set: &'a Document<TextureSet>,
    /// The layer.
    pub layer: LayerKind,
    /// The set's pack, which is also the image's.
    pub pack: &'a PackRef,
    /// The image file.
    pub image: TextureImage<'a>,
}

impl LayerImage<'_> {
    /// The finding about the image, given its bytes: an image whose header cannot be read
    /// (`LayerImageUndecodable`), or one whose channel count the layer does not take
    /// (`LayerImageFormat`: `color`, `metalness_emissive_roughness` and `normal` take 3 or 4
    /// channels, `metalness_emissive_roughness_subsurface` 4, `heightmap` 1 to 4). For either
    /// the client shows the missing texture in place of the whole set. `None` for an image
    /// that passes, and for a file without an image extension, which is not looked into.
    // Only the header is read; the client checks the decoded image.
    #[must_use]
    pub fn check(&self, bytes: &[u8]) -> Option<Diagnostic> {
        let format = self.image.format?;
        let code = match ImageInfo::read(format, bytes) {
            None => DiagnosticCode::LayerImageUndecodable,
            Some(info) => {
                let channels = info.channels?;
                if self.layer.channel_counts().contains(&channels) {
                    return None;
                }
                DiagnosticCode::LayerImageFormat
            }
        };
        let mut at = payload_at();
        at.push(self.layer.key());
        let origin = self.set.origin();
        let location = Location::new(
            Some(origin.pack.clone()),
            Some(origin.path.clone()),
            at,
            None,
        );
        Some(Diagnostic::new(
            code,
            Severity::Error,
            Phase::Validate,
            location,
        ))
    }
}

/// The file the client loads for `path`, from the stack's texture files and texture sets.
fn resolve<'a>(
    files: &'a TextureFiles,
    sets: &'a TextureSets,
    path: &TexturePath,
) -> Option<TextureFile<'a>> {
    let location = path.as_str();
    let (stack_index, from_set) = match files.source(location)? {
        TextureSource::TextureSet { stack_index } => (stack_index, true),
        TextureSource::Image { stack_index, .. } => (stack_index, false),
        // `TextureSource` is `#[non_exhaustive]`: a source this resolver does not know is no file.
        _ => return None,
    };
    let pack = files.pack(stack_index)?;
    let image = |stem: &str| files.image_in(stack_index, stem);
    let mut file = TextureFile {
        pack,
        stack_index,
        set: None,
        rejected: None,
        image: None,
    };
    if !from_set {
        file.image = image(location);
        return Some(file);
    }
    match sets.set_of(stack_index, location)? {
        Ok(set) => {
            file.set = Some(set);
            file.image = set
                .texture(LayerKind::Color)
                .and_then(|name| image(&layer_path(location, name.as_str())));
        }
        Err(rejected) => file.rejected = Some(rejected),
    }
    Some(file)
}

impl PackStack<Resource> {
    /// What the client loads for the texture path `path`: the one resolver of a texture
    /// reference (a client entity's `textures`, a particle's texture, an atlas path).
    ///
    /// The highest pack with a texture set beside the path (`<path>.texture_set.json`)
    /// competes with the highest pack that has the path itself or the path with an image
    /// extension (`.tga`, `.png`, `.jpg`, `.jpeg`, `.hdr`, `.ktx`, in that order inside a
    /// pack); the higher pack wins, and the texture set wins a tie. Texture sets of different
    /// packs are never merged. `None` when no pack has either: the client shows the missing
    /// texture and says nothing.
    #[must_use]
    pub fn resolve_texture(&self, path: &TexturePath) -> Option<TextureFile<'_>> {
        resolve(self.texture_files(), self.texture_sets(), path)
    }

    /// The image of every image layer of every loaded texture set, top pack first. Loading
    /// reads no image; a caller that wants the images' own rules checked (an image that does
    /// not decode, a channel count the layer does not take) reads these files and hands each
    /// one's bytes to [`LayerImage::check`].
    #[must_use]
    pub fn layer_images(&self) -> Vec<LayerImage<'_>> {
        let (files, sets) = (self.texture_files(), self.texture_sets());
        let mut out = Vec::new();
        for (stack_index, pack_sets) in sets.packs().iter().enumerate() {
            let Some(pack) = files.pack(stack_index) else {
                continue;
            };
            for (texture, copy) in pack_sets {
                let Ok(set) = copy else {
                    continue;
                };
                for (layer, name) in set.textures() {
                    let stem = layer_path(texture, name.as_str());
                    if let Some(image) = files.image_in(stack_index, &stem) {
                        out.push(LayerImage {
                            set,
                            layer,
                            pack,
                            image,
                        });
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use bedrock_addon_core::load::{ListedFile, LoadContext, LoadOptions, StackCx, StackPack};
    use bedrock_addon_core::pack_identity::{
        PackIdVersion, PackOrigin, PackPath, PackRef, PackType,
    };
    use bedrock_addon_core::version::SemVersion;

    use super::resolve;
    use crate::families::texture_file::load_files;
    use crate::families::texture_set::load_sets;
    use crate::id::TexturePath;

    #[test]
    fn a_subpack_s_image_is_indexed_at_its_key_and_keeps_its_path() {
        let id = PackIdVersion::new(
            uuid::Uuid::from_u128(1),
            SemVersion::new(1, 0, 0),
            PackType::Resource,
        );
        let pack = PackRef::new(0, id, "pack 1".to_owned());
        let listed = [
            ("subpacks/hd/textures/a.png", "textures/a.png"),
            // No texture file: not indexed.
            ("sounds/a.ogg", "sounds/a.ogg"),
        ]
        .map(|(path, key)| ListedFile {
            path: PackPath::new(path).unwrap(),
            key: PackPath::new(key).unwrap(),
        });
        let stack =
            [StackPack::new(pack, PackOrigin::User, LoadContext::detached()).with_files(listed)];
        let options = LoadOptions::default();
        let cx = StackCx::new(&options, &stack);
        let (files, sets) = (load_files(), load_sets());
        let files = (files.combine)(Vec::new(), &cx);
        let sets = (sets.combine)(Vec::new(), &cx);
        assert_eq!(files.packs_with("textures/a.png").count(), 1);
        assert_eq!(files.packs_with("subpacks/hd/textures/a.png").count(), 0);
        assert_eq!(files.packs_with("sounds/a.ogg").count(), 0);
        let file = resolve(&files, &sets, &TexturePath::parse("textures/a")).expect("the image");
        let image = file.image.expect("the image");
        assert_eq!(image.key, "textures/a.png");
        assert_eq!(image.path.as_str(), "subpacks/hd/textures/a.png");
    }
}
