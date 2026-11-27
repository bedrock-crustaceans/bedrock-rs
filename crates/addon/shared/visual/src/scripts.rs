//! `description.scripts`, the newest shape (format version 1.26.0 and up).

use indexmap::IndexMap;

use super::AnimateEntry;
use crate::read::{conditions, variables};
use crate::rules::NAMED_SCRIPT;
use bedrock_addon_core::molang::{Molang, MolangSource};
use bedrock_addon_core::schema::{self, Schema};

/// `scripts`: the Molang a client runs for the model, and the animations it plays.
///
/// Every member is kept as written, absent members as `None`. Vanilla keys the named scripts
/// by their lower-cased member name, so `scaleX` and `scalex` are one script; when both are
/// written the `scalex` member wins ([`Scripts::effective_scale_x`]). A named script whose value
/// is a number or a boolean passes the schema but is not stored as a script.
///
/// Format versions before 1.26.0 use
/// [`history::v1_10_0::Scripts`](super::history::v1_10_0::Scripts) and
/// [`history::v1_8_0::Scripts`](super::history::v1_8_0::Scripts).
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::molang = NAMED_SCRIPT)]
pub struct Scripts {
    /// `variables`: `variable.<name>` → `"public"`, the variables other entities may read.
    #[facet(schema::read_with = variables, skip_serializing_if = Option::is_none)]
    pub variables: Option<IndexMap<String, String>>,
    /// `initialize`: run once when the entity is created; the strings concatenated with no
    /// separator form one expression.
    #[facet(skip_serializing_if = Option::is_none)]
    pub initialize: Option<Vec<String>>,
    /// `pre_animation`: run every frame before the animations; concatenated as `initialize`.
    #[facet(skip_serializing_if = Option::is_none)]
    pub pre_animation: Option<Vec<String>>,
    /// `scale`: the model's uniform scale.
    #[facet(skip_serializing_if = Option::is_none)]
    pub scale: Option<Molang>,
    /// `scaleX`: the model's x scale (the same script as `scalex`).
    #[facet(rename = "scaleX", skip_serializing_if = Option::is_none)]
    pub scale_x: Option<Molang>,
    /// `scalex`: the model's x scale; wins over `scaleX`.
    #[facet(skip_serializing_if = Option::is_none)]
    pub scalex: Option<Molang>,
    /// `scaleY`: the model's y scale (the same script as `scaley`).
    #[facet(rename = "scaleY", skip_serializing_if = Option::is_none)]
    pub scale_y: Option<Molang>,
    /// `scaley`: the model's y scale; wins over `scaleY`.
    #[facet(skip_serializing_if = Option::is_none)]
    pub scaley: Option<Molang>,
    /// `scaleZ`: the model's z scale (the same script as `scalez`).
    #[facet(rename = "scaleZ", skip_serializing_if = Option::is_none)]
    pub scale_z: Option<Molang>,
    /// `scalez`: the model's z scale; wins over `scaleZ`.
    #[facet(skip_serializing_if = Option::is_none)]
    pub scalez: Option<Molang>,
    /// `parent_setup`: run before the model is attached to its parent.
    #[facet(skip_serializing_if = Option::is_none)]
    pub parent_setup: Option<Molang>,
    /// `should_update_bones_and_effects_offscreen`: keep animating bones and effects off
    /// screen while non-zero.
    #[facet(skip_serializing_if = Option::is_none)]
    pub should_update_bones_and_effects_offscreen: Option<Molang>,
    /// `should_update_effects_offscreen`: keep effects running off screen while non-zero.
    #[facet(skip_serializing_if = Option::is_none)]
    pub should_update_effects_offscreen: Option<Molang>,
    /// `hide_held_items`: hide held items while non-zero, overriding `enable_attachables`.
    /// Accepted from engine 1.26.30.
    #[facet(skip_serializing_if = Option::is_none)]
    pub hide_held_items: Option<Molang>,
    /// `animate`: the animations and controllers to play, by their `animations` short name.
    #[facet(schema::read_with = conditions, skip_serializing_if = Option::is_none)]
    pub animate: Option<Vec<AnimateEntry>>,
}

impl Scripts {
    /// The x-scale script vanilla stores: `scalex` when it is an expression, else `scaleX`.
    pub fn effective_scale_x(&self) -> Option<&MolangSource> {
        effective(self.scalex.as_ref(), self.scale_x.as_ref())
    }

    /// The y-scale script vanilla stores: `scaley` when it is an expression, else `scaleY`.
    pub fn effective_scale_y(&self) -> Option<&MolangSource> {
        effective(self.scaley.as_ref(), self.scale_y.as_ref())
    }

    /// The z-scale script vanilla stores: `scalez` when it is an expression, else `scaleZ`.
    pub fn effective_scale_z(&self) -> Option<&MolangSource> {
        effective(self.scalez.as_ref(), self.scale_z.as_ref())
    }
}

// Members are visited in byte order, `scaleX` before `scalex`, and each one that builds a
// script overwrites the key `scalex`, because the game stores scripts in a map.
fn effective<'a>(lower: Option<&'a Molang>, upper: Option<&'a Molang>) -> Option<&'a MolangSource> {
    lower
        .and_then(script_source)
        .or_else(|| upper.and_then(script_source))
}

// The source a named script member stores: a non-empty expression. A number or a boolean
// builds no script: anything else is read as empty and skipped.
pub(super) fn script_source(value: &Molang) -> Option<&MolangSource> {
    match value {
        Molang::Expr(source) if !source.is_empty() => Some(source),
        _ => None,
    }
}

/// Whether the place `path` (from a `description`) is a Molang member of the description
/// schema client entities and attachables share: a named script of `scripts`, the condition
/// of an object-form `render_controllers[]` or `scripts.animate[]` entry, and, where
/// `statements` says so (format versions before 1.26.0), an element of `scripts.initialize`
/// or `scripts.pre_animation`. `names_only` is the 1.8.x shape: `render_controllers` holds
/// names and there is no `scripts.animate`.
///
/// Vanilla reads a JSON number there as a Molang expression; a typed [`Molang`] member holds it as
/// an `f32`, so these are the places `json::clamp_text` reads as `Width::Float`.
#[doc(hidden)]
#[must_use]
pub fn is_molang_place(
    path: &[bedrock_addon_core::diagnostic::JsonPathSegment<'_>],
    statements: bool,
    names_only: bool,
) -> bool {
    use bedrock_addon_core::diagnostic::JsonPathSegment::{Index, Key};
    match path {
        [Key("scripts"), Key(name)] => matches!(
            *name,
            "scale"
                | "scaleX"
                | "scalex"
                | "scaleY"
                | "scaley"
                | "scaleZ"
                | "scalez"
                | "parent_setup"
                | "should_update_bones_and_effects_offscreen"
                | "should_update_effects_offscreen"
                | "hide_held_items"
        ),
        [
            Key("scripts"),
            Key("initialize" | "pre_animation"),
            Index(_),
        ] => statements,
        [Key("scripts"), Key("animate"), Index(_), Key(_)]
        | [Key("render_controllers"), Index(_), Key(_)] => !names_only,
        _ => false,
    }
}
