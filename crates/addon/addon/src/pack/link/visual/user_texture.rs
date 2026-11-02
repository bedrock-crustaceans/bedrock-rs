//! `description.textures` of a client entity, an attachable or an atomic client entity: each
//! value is a texture path, handed to the texture group as written.

use bedrock_addon_core::diagnostic::{Severity, VanillaText};

use super::super::{At, Findings, LinkCx, kinds, texture_not_found};
use super::user::User;

/// The texture-set loader's line for an image no pack has; the texture's path.
// The texture-set loader's line for an image no pack has, logged at the warning level.
const NOT_FOUND: &str = "Failed to find either texture set or image file of name ({})";

/// `textures`: each value is a texture path some pack of the stack has, as written.
/// A texture set that did not load was reported by the load and is not reported again. A
/// server-side link (`LinkOptions::client` false) says nothing.
pub(super) fn check(cx: &LinkCx<'_>, out: &mut Findings, user: &User<'_>) {
    // A dedicated server has no client entity, attachable or texture-set loader: it never
    // reads these textures.
    if !cx.client() {
        return;
    }
    for (key, value) in user.textures.into_iter().flatten() {
        // The name of an atlas is bound by the renderer, not read from a pack.
        if is_atlas_name(value.as_str()) || cx.texture_file(value).is_some() {
            continue;
        }
        // A texture of the game's own, when a host answers for them.
        if cx
            .built_ins()
            .is_some_and(|host| host.contains(kinds::TEXTURE, value.as_str()))
        {
            continue;
        }
        // The client starts the image's load when it reads the definition at resource load,
        // logs the content-log warning and goes on with the missing texture: a warning with
        // its text.
        let at = At::new(user.origin, user.entry("textures", key), Severity::Warning).with_text(
            VanillaText::Formatted {
                template: NOT_FOUND,
                args: vec![value.as_str().to_owned()],
            },
        );
        out.push(texture_not_found(at, value.as_str()));
    }
}

/// Whether `name` is the resource name of a texture atlas, not a path of a pack: it has the
/// prefix of the names the atlases of the texture family carry (`atlas.terrain`,
/// `atlas.items`), as the `atlas.shield` of the game's own shield attachable has.
fn is_atlas_name(name: &str) -> bool {
    // The prefix is read off the family's own name, so that there is one spelling of it.
    let atlas = crate::families::texture_atlas::AtlasKind::Terrain.expected_name();
    atlas.split_once('.').is_some_and(|(prefix, _)| {
        name.strip_prefix(prefix)
            .is_some_and(|rest| rest.starts_with('.'))
    })
}
