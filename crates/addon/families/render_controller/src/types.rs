//! The family's types and their constructors: the game's defaults for the absent members
//! and the bone-pattern conversion.

use indexmap::IndexMap;

use super::fold_case;
use bedrock_addon_core::molang::{Molang, MolangSource, RawVersion};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_shared_shapes::TrueOnly;

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
    pub filter_lighting: Option<TrueOnly>,
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

/// The Molang version the game compiles its own default expressions with, whatever the pack.
const BUILT_IN: RawVersion = RawVersion(13);

/// The default green channel of `on_fire_color`.
pub const ON_FIRE_GREEN: &str =
    "math.lerp(0.3f, 0.15f, math.pow(math.sin((query.on_fire_time / 20.0f) * 360.0f), 2.0f))";
/// The default alpha channel of `on_fire_color`.
pub const ON_FIRE_ALPHA: &str = "math.hermite_blend(math.clamp((query.is_on_fire ? query.on_fire_time : (((query.on_fire_time < 5.0f) ? (5.0f - query.on_fire_time) : 0.0f))) / 5.0f, 0.0f, 1.0f)) * 0.7";

fn built_in(text: &'static str) -> Molang {
    Molang::Expr(MolangSource::string(text, BUILT_IN))
}

impl RenderControllerFile {
    /// An empty file at format 1.8.0, the version the game's own files use.
    pub fn new() -> Self {
        Self {
            format_version: known::V1_8_0,
            render_controllers: IndexMap::new(),
            dropped: IndexMap::new(),
        }
    }

    /// The controller whose key is `name` after case folding (the game folds the whole file).
    pub fn get(&self, name: &str) -> Option<&RenderController> {
        self.render_controllers.get(fold_case(name).as_str())
    }
}

impl Default for RenderControllerFile {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderController {
    /// A controller with the two required members and every other member absent.
    pub fn new(geometry: MolangSource, textures: Vec<MolangSource>) -> Self {
        Self {
            geometry,
            textures,
            materials: Vec::new(),
            part_visibility: Vec::new(),
            arrays: None,
            color: None,
            overlay_color: None,
            on_fire_color: None,
            is_hurt_color: None,
            uv_anim: None,
            light_color_multiplier: None,
            ignore_lighting: None,
            filter_lighting: None,
            rebuild_animation_matrices: None,
        }
    }

    /// The version a channel a colour object leaves out is compiled with: the pack's, which
    /// every source of the file carries.
    fn context(&self) -> RawVersion {
        self.geometry.raw_version().unwrap_or(BUILT_IN)
    }

    fn rgba(&self, written: Option<&Rgba>, default: impl FnOnce() -> [Molang; 4]) -> [Molang; 4] {
        written.map_or_else(default, |color| color.channels(self.context()))
    }

    /// `color` as the game uses it (r, g, b, a): absent, `this` in every channel, which is 0.
    pub fn color(&self) -> [Molang; 4] {
        self.rgba(self.color.as_ref(), || [(); 4].map(|()| built_in("this")))
    }

    /// `overlay_color` as the game uses it: absent, `this` in every channel.
    pub fn overlay_color(&self) -> [Molang; 4] {
        self.rgba(self.overlay_color.as_ref(), || {
            [(); 4].map(|()| built_in("this"))
        })
    }

    /// `on_fire_color` as the game uses it: absent, red 0.8, the flicker [`ON_FIRE_GREEN`],
    /// blue 0 and the fade [`ON_FIRE_ALPHA`], compiled with Molang version 13.
    pub fn on_fire_color(&self) -> [Molang; 4] {
        self.rgba(self.on_fire_color.as_ref(), || {
            [
                Molang::Const(0.8),
                built_in(ON_FIRE_GREEN),
                Molang::Const(0.0),
                built_in(ON_FIRE_ALPHA),
            ]
        })
    }

    /// `is_hurt_color` as the game uses it: absent, (1, 0, 0, 0.25).
    pub fn is_hurt_color(&self) -> [Molang; 4] {
        self.rgba(self.is_hurt_color.as_ref(), || {
            [
                Molang::Const(1.0),
                Molang::Const(0.0),
                Molang::Const(0.0),
                Molang::Const(0.25),
            ]
        })
    }

    /// `uv_anim` with its default: offset (0, 0), scale (1, 1).
    pub fn uv_anim(&self) -> UvAnim {
        self.uv_anim.clone().unwrap_or(UvAnim {
            offset: [Molang::Const(0.0), Molang::Const(0.0)],
            scale: [Molang::Const(1.0), Molang::Const(1.0)],
        })
    }

    /// `light_color_multiplier` with its default, the constant 1.
    pub fn light_color_multiplier(&self) -> Molang {
        self.light_color_multiplier
            .clone()
            .unwrap_or(Molang::Const(1.0))
    }

    /// `ignore_lighting` with its default, false.
    pub fn ignore_lighting(&self) -> bool {
        self.ignore_lighting.unwrap_or(false)
    }

    /// `filter_lighting`: true when written (only `true` may be), else false.
    pub fn filter_lighting(&self) -> bool {
        self.filter_lighting.is_some()
    }

    /// `rebuild_animation_matrices` with its default, false.
    pub fn rebuild_animation_matrices(&self) -> bool {
        self.rebuild_animation_matrices.unwrap_or(false)
    }
}

impl Rgba {
    /// The four channels (r, g, b, a) of a written colour object: a channel it leaves out is
    /// the expression `this`, compiled like the file's other expressions with the pack's Molang
    /// version `context`.
    pub fn channels(&self, context: RawVersion) -> [Molang; 4] {
        [&self.r, &self.g, &self.b, &self.a].map(|channel| {
            channel
                .clone()
                .unwrap_or_else(|| Molang::Expr(MolangSource::string("this", context)))
        })
    }
}

impl BonePattern {
    /// The pattern as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The regular expression the game matches bone names with: each `*` becomes
    /// `[a-zA-Z0-9_\.]*` and every other character is copied as it is, so a `.` of the pattern
    /// matches any character.
    pub fn to_regex(&self) -> String {
        self.0.replace('*', r"[a-zA-Z0-9_\.]*")
    }
}
