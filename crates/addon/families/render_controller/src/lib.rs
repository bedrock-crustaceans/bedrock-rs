//! Render controllers: `render_controllers/**/*.json` of a resource pack. One file holds any
//! number of controllers under `render_controllers`; client entities and attachables name them.
//! Vanilla reads them with an ad-hoc loader behind one strict schema built at 1.8.0 whatever the
//! file's `format_version` says, so there is one struct and no upgrade step.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod fold;
mod load;
mod molang;
mod new;
mod read;
mod validate;
mod write;

#[cfg(test)]
mod tests;

use indexmap::IndexMap;

use bedrock_addon_core::molang::{Molang, MolangSource};
use bedrock_addon_core::version::FormatVersion;

#[doc(hidden)]
pub use load::load_registry;

pub use fold::fold_case;
pub use new::{ON_FIRE_ALPHA, ON_FIRE_GREEN};

/// A render controller file (`render_controllers/*.json`): `{"format_version",
/// "render_controllers": {"controller.render.<name>": {…}}}`.
///
/// One schema serves every `format_version` from 1.8.0 up to 65535.0.0; the version selects
/// nothing. Before the file is read its whole text is lower-cased outside single-quoted spans
/// ([`fold_case`]), so every key and every string of the model is lower case except Molang
/// string literals. Any member the schema does not declare, a missing required member or a
/// value of the wrong JSON type drops the whole file, as the game does.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "render_controllers")]
pub struct RenderControllerFile {
    /// `format_version`: the version the file was written in (at least 1.8.0).
    pub format_version: FormatVersion,
    /// `render_controllers`: each controller by its key (`controller.render.<name>`), case
    /// folded, in the game's member order (sorted keys).
    pub render_controllers: IndexMap<String, RenderController>,
    /// The controllers of the file the game drops when it loads it (no `textures`, Molang in
    /// `geometry`, `textures` or `arrays` that does not compile, or a `materials` value
    /// written as a number, which the game reads as the empty text and does not compile
    /// either), by key, as they were written: the game registers none of them, and the file is written back with them, so a
    /// written file reads back to the same controllers and the same findings. Empty in a file
    /// every controller of which loads. A key is in one of the two maps: a file holding one
    /// key in both is not written (`NotRepresentable` at `render_controllers/<key>`).
    pub dropped: IndexMap<String, RenderController>,
}

/// One render controller: which geometry, materials and textures a model is drawn with, which
/// bones are visible, and how it is tinted and lit.
///
/// Only `geometry` and `textures` are required. The absent members have the game's defaults,
/// which the accessors of the same name give (`color()`, `is_hurt_color()`, …).
#[derive(Clone, Debug, PartialEq)]
pub struct RenderController {
    /// `geometry`: a Molang expression naming the geometry (`geometry.default`, or an
    /// `array.<name>[…]` lookup).
    pub geometry: MolangSource,
    /// `textures`: one expression per texture layer, in order; never empty.
    pub textures: Vec<MolangSource>,
    /// `materials`: bone pattern and material expression, in the order the game applies them:
    /// the elements in array order and, inside one element object, sorted by key. A later entry
    /// wins for a bone several patterns match. Empty when the member is absent. A JSON bool is
    /// kept as the text `true` / `false` and a JSON number as the empty text, as the game reads
    /// them.
    pub materials: Vec<(BonePattern, MolangSource)>,
    /// `part_visibility`: bone pattern and visibility, in the same order as `materials`; a
    /// later entry wins. Empty when the member is absent.
    pub part_visibility: Vec<(BonePattern, Molang)>,
    /// `arrays`: the named arrays the expressions index.
    pub arrays: Option<RenderControllerArrays>,
    /// `color`: the tint; absent, every channel is `this`.
    pub color: Option<Rgba>,
    /// `overlay_color`: the overlay; absent, every channel is `this`.
    pub overlay_color: Option<Rgba>,
    /// `on_fire_color`: the overlay while burning; absent, the game's flicker expression.
    pub on_fire_color: Option<Rgba>,
    /// `is_hurt_color`: the overlay while hurt; absent, red at a quarter opacity.
    pub is_hurt_color: Option<Rgba>,
    /// `uv_anim`: the texture offset and scale.
    pub uv_anim: Option<UvAnim>,
    /// `light_color_multiplier`: a factor on the light colour; absent 1.
    pub light_color_multiplier: Option<Molang>,
    /// `ignore_lighting`: draw without lighting; absent false.
    pub ignore_lighting: Option<bool>,
    /// `filter_lighting`: only `true` may be written; absent false.
    pub filter_lighting: Option<FilterLighting>,
    /// `rebuild_animation_matrices`: absent false.
    pub rebuild_animation_matrices: Option<bool>,
}

/// The `arrays` of a controller: lists of resource references by name, one map per kind. A
/// kind written without any array fails the file, so an empty map is an absent kind.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderControllerArrays {
    /// `geometries`: each `array.<name>` and its geometry references; no list is empty.
    pub geometries: IndexMap<String, Vec<MolangSource>>,
    /// `materials`: each `array.<name>` and its material references.
    pub materials: IndexMap<String, Vec<MolangSource>>,
    /// `textures`: each `array.<name>` and its texture references.
    pub textures: IndexMap<String, Vec<MolangSource>>,
}

/// A colour object (`color`, `overlay_color`, `on_fire_color`, `is_hurt_color`): one Molang
/// value per channel.
///
/// Once the object is written, a channel it leaves out is `this` (which is 0 for a colour), not
/// the member's default: [`Rgba::channels`] gives the four values the game uses.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rgba {
    /// `r`.
    pub r: Option<Molang>,
    /// `g`.
    pub g: Option<Molang>,
    /// `b`.
    pub b: Option<Molang>,
    /// `a`.
    pub a: Option<Molang>,
}

/// `uv_anim`: both members are required, each exactly two values (u, v).
#[derive(Clone, Debug, PartialEq)]
pub struct UvAnim {
    /// `offset`: added to the texture coordinates; the game's default is (0, 0).
    pub offset: [Molang; 2],
    /// `scale`: multiplied into the texture coordinates; the game's default is (1, 1).
    pub scale: [Molang; 2],
}

/// A bone pattern of `materials` and `part_visibility`: letters, digits, `_`, `.`, `:` and
/// `*`, where `*` stands for any run of bone-name characters.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BonePattern(pub String);

/// The one value `filter_lighting` may have: `true`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FilterLighting;
