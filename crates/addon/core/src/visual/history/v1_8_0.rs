//! Format versions 1.8.0 up to 1.10.0.

use crate::molang::Molang;

/// `scripts` at format versions 1.8.0 up to 1.10.0: `pre_animation`, the scale scripts and
/// `parent_setup` only. `variables`, `initialize`, `animate` and the off-screen scripts do not
/// exist yet.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
pub struct Scripts {
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
}
