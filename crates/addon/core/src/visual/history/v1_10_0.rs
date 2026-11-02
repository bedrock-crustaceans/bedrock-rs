//! Format versions 1.10.0 up to 1.26.0.

use indexmap::IndexMap;

use crate::json::StrictString;
use crate::molang::Molang;
use crate::visual::AnimateEntry;

/// `scripts` at format versions 1.10.0 up to 1.26.0: as [`crate::visual::Scripts`], except
/// that each `initialize` and `pre_animation` element is a Molang value of its own and
/// `hide_held_items` does not exist.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
pub struct Scripts {
    /// `variables`: `variable.<name>` → `"public"`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub variables: Option<IndexMap<String, StrictString>>,
    /// `initialize`: each element a Molang value.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub initialize: Option<Vec<Molang>>,
    /// `pre_animation`: each element a Molang value.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub pre_animation: Option<Vec<Molang>>,
    /// `scale`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub scale: Option<Molang>,
    /// `scaleX` (the same script as `scalex`).
    #[facet(default, rename = "scaleX", skip_serializing_if = Option::is_none)]
    pub scale_x: Option<Molang>,
    /// `scalex`; wins over `scaleX`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub scalex: Option<Molang>,
    /// `scaleY` (the same script as `scaley`).
    #[facet(default, rename = "scaleY", skip_serializing_if = Option::is_none)]
    pub scale_y: Option<Molang>,
    /// `scaley`; wins over `scaleY`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub scaley: Option<Molang>,
    /// `scaleZ` (the same script as `scalez`).
    #[facet(default, rename = "scaleZ", skip_serializing_if = Option::is_none)]
    pub scale_z: Option<Molang>,
    /// `scalez`; wins over `scaleZ`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub scalez: Option<Molang>,
    /// `parent_setup`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub parent_setup: Option<Molang>,
    /// `should_update_bones_and_effects_offscreen`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub should_update_bones_and_effects_offscreen: Option<Molang>,
    /// `should_update_effects_offscreen`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub should_update_effects_offscreen: Option<Molang>,
    /// `animate`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub animate: Option<Vec<AnimateEntry>>,
}
