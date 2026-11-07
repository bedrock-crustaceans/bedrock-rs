//! The references of the deferred settings that the link pass resolves (rules R51 and R52):
//! a checker for `StackSet::link` to call with the block registry of the behaviour stack and
//! the texture files of the resource stack. Every finding is a kept error in `Phase::Link`.
//!
//! The identifiers themselves (R47) are checked from the other side: the client-biome
//! components that name them resolve against `DeferredSettings::{lighting, water, atmosphere,
//! color_grading, cubemaps}`; an empty name is accepted, and a biome without the component uses
//! `DeferredSettings::defaults`.

use super::DeferredSettings;
use super::codes;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Phase,
};
use bedrock_addon_core::document::Origin;
use bedrock_addon_core::id::BlockId;

/// What a caustics texture path names in the resource stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Texture {
    /// No pack has an image at the path.
    Missing,
    /// An image that cannot be decoded.
    Undecodable,
    /// An image of this size in pixels.
    Image {
        /// The width.
        width: u32,
        /// The height.
        height: u32,
    },
}

/// What the references of the deferred settings resolve against outside the group.
pub trait Targets {
    /// A block the behaviour stack registers, or a built-in one.
    fn block(&self, id: &BlockId) -> bool;
    /// The image at the texture path `path` (a pack path without its extension).
    fn texture(&self, path: &str) -> Texture;
}

/// A link finding about the value at `path` (the members from the root) of the document at
/// `origin`, naming what it refers to: a kept error, as both callers mirror a level-3 line
/// after which the game goes on.
fn kept_error(
    code: DiagnosticCode,
    origin: &Origin,
    path: &[&str],
    kind: &'static str,
    id: &str,
) -> Diagnostic {
    Diagnostic::error(code)
        .at(JsonPath::new().with(path))
        .in_file(origin)
        .phase(Phase::Link)
        .into_kept()
        .detail(DiagnosticDetail::Reference {
            kind,
            id: id.to_owned(),
            hint: None,
        })
}

/// The link findings of `settings`, local lights first (the order the entries merged in),
/// then the water settings in load order.
///
/// - A local light whose key is no block: `UnresolvedReference`, "%s is not a block"; the
///   game drops the entry.
/// - A caustics texture no pack has: `TextureNotFound` with the game's text; the game uses
///   its own `renderer/caustics`.
/// - A caustics texture that cannot be decoded, or whose height is no multiple of its width
///   (the frames are stacked vertically): `InvalidTexture` with the game's text.
///
/// The game loads the caustics texture of the default water settings only; every effective
/// definition is checked here, since which one is the default depends on the stack.
pub fn check(settings: &DeferredSettings, targets: &dyn Targets) -> Diagnostics {
    let mut d = Diagnostics::default();
    for block in settings.local_lighting.lights.keys() {
        if targets.block(block) {
            continue;
        }
        let Some(origin) = settings.local_lighting.origin(block) else {
            continue;
        };
        // The game logs the line at the error level and goes on to the next entry, so the
        // finding is a kept error.
        d.push(kept_error(
            codes::NOT_A_BLOCK.with([block.as_str()]).into(),
            origin,
            &["minecraft:local_light_settings", block.as_str()],
            "block",
            block.as_str(),
        ));
    }
    for (_, document) in &settings.water {
        let Some(texture) = document
            .caustics
            .as_ref()
            .and_then(|caustics| caustics.texture.as_deref())
        else {
            continue;
        };
        let code = match targets.texture(texture) {
            Texture::Missing => &codes::CAUSTICS_NOT_FOUND,
            // An undecodable image and one that does not divide into frames, each with the
            // game's text.
            Texture::Undecodable => &codes::CAUSTICS_UNDECODABLE,
            Texture::Image { width, height } if width == 0 || height % width != 0 => {
                &codes::CAUSTICS_FRAMES
            }
            Texture::Image { .. } => continue,
        };
        // The game logs the chosen text at the error level and falls back to
        // "renderer/caustics", so the finding is a kept error.
        d.push(kept_error(
            code.with([texture]).into(),
            document.origin(),
            &["minecraft:water_settings", "caustics", "texture"],
            "texture",
            texture,
        ));
    }
    d
}
