//! Format versions 1.10.0 up to 1.26.0.

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, JsonPath, Phase};
use bedrock_addon_core::molang::{Molang, MolangFields, MolangSource, MolangVisitor};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::UpgradeCx;

use crate::AnimateEntry;
use crate::rules::{CONDITION, NAMED_SCRIPT, visit_animate};

/// The version from which vanilla stops lower-casing the named scripts: a document written
/// below it has them lower-cased when it loads.
const NO_LOWER_CASING: FormatVersion = FormatVersion::new(1, 18, 20);

/// `scripts` at format versions 1.10.0 up to 1.26.0: as [`crate::Scripts`], except
/// that each `initialize` and `pre_animation` element is a Molang value of its own and
/// `hide_held_items` does not exist.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
pub struct Scripts {
    /// `variables`: `variable.<name>` → `"public"`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub variables: Option<IndexMap<String, String>>,
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

/// The named scripts, the `initialize` and `pre_animation` elements and the `animate`
/// conditions, as the 1.10 schema types them (the walk enters `scripts` first).
impl MolangFields for Scripts {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.field("initialize", &NAMED_SCRIPT, &mut self.initialize);
        visitor.field("pre_animation", &NAMED_SCRIPT, &mut self.pre_animation);
        visitor.field("scale", &NAMED_SCRIPT, &mut self.scale);
        visitor.field("scaleX", &NAMED_SCRIPT, &mut self.scale_x);
        visitor.field("scalex", &NAMED_SCRIPT, &mut self.scalex);
        visitor.field("scaleY", &NAMED_SCRIPT, &mut self.scale_y);
        visitor.field("scaley", &NAMED_SCRIPT, &mut self.scaley);
        visitor.field("scaleZ", &NAMED_SCRIPT, &mut self.scale_z);
        visitor.field("scalez", &NAMED_SCRIPT, &mut self.scalez);
        visitor.field("parent_setup", &NAMED_SCRIPT, &mut self.parent_setup);
        visitor.field(
            "should_update_bones_and_effects_offscreen",
            &NAMED_SCRIPT,
            &mut self.should_update_bones_and_effects_offscreen,
        );
        visitor.field(
            "should_update_effects_offscreen",
            &NAMED_SCRIPT,
            &mut self.should_update_effects_offscreen,
        );
        visit_animate(self.animate.as_mut(), &CONDITION, visitor);
    }
}

/// D4: whether the 1.10 → 1.26 step tells of a named script whose quoted literal the
/// lower-casing changes. The client entity's step tells it, the attachable's folds silently
/// (both pinned in the families' tests); which is the game's is not decided here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoldedLiterals {
    /// A changed literal is told, with a `ValueRewritten` information at the script.
    Told,
    /// A changed literal is folded and nothing is said.
    Silent,
}

impl Scripts {
    /// The step to the newest `scripts` (format version 1.26.0). Vanilla rewrites nothing at
    /// 1.26.0; the version selects another schema. So the step is the typed conversion and
    /// the one rule the loader keys on the original version:
    /// - each `initialize` and `pre_animation` element, a Molang value of its own until now,
    ///   is checked as one (a rejected element fails the document, as the 1.10 schema does;
    ///   with the `molang` feature) and becomes a plain string; an element that is a number or
    ///   a boolean, which the loader skips, is dropped with an information;
    /// - in a document written below 1.18.20 the text of every named script is lower-cased,
    ///   as the loader does before it parses them, and `literals` says whether a changed
    ///   quoted literal is told. `animate` conditions and variable names stay as written.
    ///
    /// `at` is the `scripts` member.
    #[must_use]
    pub fn upgrade(
        self,
        literals: FoldedLiterals,
        at: &JsonPath,
        cx: &mut UpgradeCx<'_>,
    ) -> crate::Scripts {
        // Named scripts are lower-cased below 1.18.20, as the game does when it loads them.
        let fold = Fold {
            fold: cx.original_version < NO_LOWER_CASING,
            tell: literals == FoldedLiterals::Told,
        };
        let s = self;
        crate::Scripts {
            variables: s.variables,
            initialize: elements(s.initialize, "initialize", fold, at, cx),
            pre_animation: elements(s.pre_animation, "pre_animation", fold, at, cx),
            scale: named(s.scale, "scale", fold, at, cx),
            scale_x: named(s.scale_x, "scaleX", fold, at, cx),
            scalex: named(s.scalex, "scalex", fold, at, cx),
            scale_y: named(s.scale_y, "scaleY", fold, at, cx),
            scaley: named(s.scaley, "scaley", fold, at, cx),
            scale_z: named(s.scale_z, "scaleZ", fold, at, cx),
            scalez: named(s.scalez, "scalez", fold, at, cx),
            parent_setup: named(s.parent_setup, "parent_setup", fold, at, cx),
            should_update_bones_and_effects_offscreen: named(
                s.should_update_bones_and_effects_offscreen,
                "should_update_bones_and_effects_offscreen",
                fold,
                at,
                cx,
            ),
            should_update_effects_offscreen: named(
                s.should_update_effects_offscreen,
                "should_update_effects_offscreen",
                fold,
                at,
                cx,
            ),
            hide_held_items: None,
            animate: s.animate,
        }
    }
}

/// Whether the step lower-cases (`fold`), and whether it tells a changed literal (`tell`).
#[derive(Clone, Copy)]
struct Fold {
    fold: bool,
    tell: bool,
}

/// The elements of `initialize` or `pre_animation` as the strings 1.26.0 holds.
fn elements(
    list: Option<Vec<Molang>>,
    name: &str,
    fold: Fold,
    at: &JsonPath,
    cx: &mut UpgradeCx<'_>,
) -> Option<Vec<String>> {
    let Fold { fold, tell } = fold;
    let list = list?;
    let mut strings = Vec::with_capacity(list.len());
    for (index, element) in list.into_iter().enumerate() {
        let mut element_at = at.with(&[name]);
        element_at.push_index(index);
        if let Molang::Expr(source) = element {
            // The schema's `ExpressionNode` parses each element as written.
            crate::rules::check_element(&source, &element_at, cx);
            if tell && fold && has_cased_literal(source.as_str()) {
                cx.diagnostics.push(literal_folded(element_at));
            }
            strings.push(folded(source.as_str(), fold));
        } else {
            // The game joins the string elements and skips the ones that are not strings.
            cx.diagnostics.push(
                Diagnostic::info(DiagnosticCode::DroppedField { text: None })
                    .at(element_at)
                    .phase(Phase::Upgrade),
            );
        }
    }
    Some(strings)
}

/// A named script with its text lower-cased when `fold` is set. A number or a boolean, which
/// builds no script, stays.
fn named(
    value: Option<Molang>,
    name: &str,
    fold: Fold,
    at: &JsonPath,
    cx: &mut UpgradeCx<'_>,
) -> Option<Molang> {
    let Fold { fold, tell } = fold;
    match value {
        Some(Molang::Expr(source)) if fold => {
            let lower = folded(source.as_str(), true);
            if lower == source.as_str() {
                return Some(Molang::Expr(source));
            }
            if tell && has_cased_literal(source.as_str()) {
                cx.diagnostics.push(literal_folded(at.with(&[name])));
            }
            Some(Molang::Expr(match source.raw_version() {
                Some(version) => MolangSource::string(lower, version),
                None => MolangSource::string_without_context(lower),
            }))
        }
        other => other,
    }
}

/// Whether a script's text has a single-quoted literal with an upper-case ASCII letter: the
/// one part of it the lower-casing changes the meaning of (Molang reads names in any case).
fn has_cased_literal(text: &str) -> bool {
    text.split('\'')
        .skip(1)
        .step_by(2)
        .any(|literal| literal.bytes().any(|byte| byte.is_ascii_uppercase()))
}

/// The note that the script at `at` is held and written with a literal in another case than
/// the file's: the game's own lower-casing, reported as an information because the game
/// discards the original case itself.
fn literal_folded(at: JsonPath) -> Diagnostic {
    Diagnostic::info(DiagnosticCode::ValueRewritten { text: None })
        .at(at)
        .phase(Phase::Upgrade)
}

/// The whole text lower-cased, quoted literals included, as the game does it.
fn folded(text: &str, fold: bool) -> String {
    if fold {
        text.to_ascii_lowercase()
    } else {
        text.to_owned()
    }
}
