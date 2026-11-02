//! A particle effect's texture is a texture file of the resource stack.
//!
//! The game loads the texture when the effect is first drawn; a path no pack has an image at
//! draws the missing texture and nothing is said at load. Told here at `texture`, with the
//! path as written, as the linked-asset check's line: an error.
//!
//! The crate knows the textures of the stack's packs only: a texture of the game's own
//! resource pack (`textures/particle/particles`) is missing unless that pack is on the stack.
// `PackStack::resolve_texture` is the one resolver of a texture path. The presence of the
// member is the load's to tell.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::id::TexturePath;

use super::super::{At, Findings, LinkCx, Validator, kinds, texture_not_found};
use super::particle::payload;
use crate::families::texture_atlas::AtlasKind;

/// Whether `texture` is the resource name of one of the two atlases (`atlas.terrain`,
/// `atlas.items`), byte for byte: a particle of a block or of an item is drawn from the atlas
/// and names it so, where another effect has a path. The game always has both atlases,
/// whatever the stack's packs add to them.
// The game's own effects `particles/block_destruct.json`, `block_slide.json`,
// `breaking_item_terrain.json`, `breeze_ground_particle.json`, `smash_ground_particle.json`
// (`atlas.terrain`) and `breaking_item_icon.json` (`atlas.items`) name an atlas, and no pack
// has a file of either name.
fn is_atlas(texture: &str) -> bool {
    [AtlasKind::Terrain, AtlasKind::Item]
        .iter()
        .any(|atlas| atlas.expected_name() == texture)
}

/// `description.basic_render_parameters.texture` of every effect is a path the client
/// finds a texture at, or the name of one of the two atlases. One it finds none at is a
/// `TextureNotFound` error at the member. An empty path is no reference (the load has told
/// the missing texture), and a texture set that did not load was told when it was read.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    for (_, document) in cx.resource().particles() {
        let texture = &document.value().description.basic_render_parameters.texture;
        if texture.is_empty() || is_atlas(texture) {
            continue;
        }
        if cx
            .texture_file(&TexturePath::parse(texture.as_str()))
            .is_none()
            && !cx.host_has(kinds::TEXTURE, without_extension(texture))
        {
            let mut path = payload();
            path.push("description");
            path.push("basic_render_parameters");
            path.push("texture");
            // The linked-asset check names it without its extension (`font/test_font` for
            // `font/test_font.png`); its line is an error from that check, written as such.
            let at = At::new(document.origin(), path, Severity::Error).validator(
                Validator::InvalidPath,
                1,
                without_extension(texture),
            );
            out.push(texture_not_found(at, texture));
        }
    }
}

/// `path` without an image file's extension.
fn without_extension(path: &str) -> &str {
    [".png", ".tga", ".jpg", ".jpeg"]
        .iter()
        .find_map(|extension| {
            let cut = path.len().checked_sub(extension.len())?;
            path.get(cut..)
                .filter(|tail| tail.eq_ignore_ascii_case(extension))
                .map(|_| &path[..cut])
        })
        .unwrap_or(path)
}
