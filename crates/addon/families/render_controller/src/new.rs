//! Constructors and the inherent helpers: the game's defaults for the absent members
//! and the bone-pattern conversion.

use indexmap::IndexMap;

use super::{BonePattern, RenderController, RenderControllerFile, Rgba, UvAnim, fold_case};
use bedrock_addon_core::molang::{Molang, MolangSource, RawVersion};
use bedrock_addon_core::version::FormatVersion;

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
            format_version: FormatVersion::new(1, 8, 0),
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
