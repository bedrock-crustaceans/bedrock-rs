//! The loader's rules on the newest shared description members ([`VisualSlots::validate`]),
//! the Molang field specs of the members, for client entities and attachables alike.

#[cfg(feature = "molang")]
use bedrock_addon_core::diagnostic::Location;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, Phase, Severity,
};
#[cfg(feature = "molang")]
use bedrock_addon_core::molang::{MolangFailure, MolangSource};
use bedrock_addon_core::molang::{MolangFieldSpec, ReaderKind};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::{UpgradeCx, ValidateCx};
use indexmap::IndexMap;

use crate::{AnimateEntry, HeldItemScale, VisualSlots, codes};

/// The engine version that first declares `scripts.hide_held_items` and reads
/// `held_item_scale` as Molang. Before it the member is a plain float.
pub(crate) const ENGINE_1_26_30: FormatVersion = known::V1_26_30;

/// A named script as the schema types it (`ExpressionNode`): a number, a boolean or a string
/// at the pack's version. A string the parser rejects fails the schema, so the document is refused
/// and the file is skipped, in the 1.8 and 1.10 shapes too.
pub const NAMED_SCRIPT: MolangFieldSpec =
    MolangFieldSpec::REJECTING.reader(ReaderKind::SchemaValidated);

/// The condition of an object-form `render_controllers` or `scripts.animate` entry: typed as
/// a named script is.
pub const CONDITION: MolangFieldSpec = NAMED_SCRIPT;

/// `held_item_scale` in its Molang form (engine 1.26.30 and up): compiled with the pack's
/// version. The 1.26 schema types it as an expression child, which fails when the parser
/// rejects the text, and the game skips a file that fails its schema: the parser's lines are
/// errors and the document is refused (an error after which the game drops the document).
/// The same check fails an array; the typed field (`HeldItemScale`, a number or Molang)
/// refuses one before this spec is asked, so the reader's array form is never used.
pub const HELD_ITEM_SCALE: MolangFieldSpec = MolangFieldSpec::REJECTING
    .reader(ReaderKind::ScalarOrArray)
    .compile_empty();

/// `initialize` / `pre_animation` as the loader parses them: the strings joined into one
/// text. A rejected text drops that script; the file loads.
#[cfg(feature = "molang")]
// The parser's lines come before "Failed to parse"; the definition loads on without the
// script, so these are kept errors: error-level findings after which the game keeps the
// document.
const LOADER_SCRIPT: MolangFieldSpec = MolangFieldSpec::KEPT
    .reader(ReaderKind::SchemaValidated)
    .on_failure(MolangFailure::DropEntry);

/// Stands for the file's path in a text until the load knows it.
pub const FILE: &str = "\u{0}file";

/// The compile check of one `initialize` / `pre_animation` element of a 1.8 or 1.10 document,
/// as its schema parses it: on its own, as written. A rejection is an error and fails the
/// document. Nothing without the `molang` feature or a compile check.
#[cfg_attr(not(feature = "molang"), allow(unused_variables))]
pub(crate) fn check_element(
    source: &bedrock_addon_core::molang::MolangSource,
    at: &JsonPath,
    cx: &mut UpgradeCx<'_>,
) {
    #[cfg(feature = "molang")]
    if let Some(check) = &cx.load.molang {
        let molang = molang_context(cx.load, cx.original_version);
        let location = Location::new(None, None, at.clone(), None);
        // The load's first Molang pass gave the element the pack's version; a struct built
        // another way gets it here.
        let versioned;
        let source = if source.raw_version().is_some() {
            source
        } else {
            versioned = MolangSource::string(source.as_str(), cx.load.molang_version);
            &versioned
        };
        check.check_source(
            source,
            &NAMED_SCRIPT,
            bedrock_addon_core::molang::Side::Client,
            &molang,
            &location,
            cx.diagnostics,
        );
    }
}

/// The Molang context of a document of `version` in `load`'s pack.
#[cfg(feature = "molang")]
fn molang_context(
    load: &bedrock_addon_core::load::LoadContext,
    version: bedrock_addon_core::version::FormatVersion,
) -> bedrock_addon_core::molang::MolangContext {
    bedrock_addon_core::molang::MolangContext::new(
        load.min_engine_version.as_ref(),
        load.molang_experiments,
        &bedrock_addon_core::version::FormatVersionInput {
            version: Some(version),
            beta_literal: false,
            prerelease: false,
        },
        Some(load.molang_version),
    )
}

impl VisualSlots {
    /// The loader's rules on the newest shape, after the schema and the generic Molang check;
    /// `at` is the description:
    /// - `held_item_scale` as a number at or below 0 is not used (a kept error with vanilla's
    ///   text, the file's path standing as [`FILE`] until the load knows it);
    /// - `initialize` and `pre_animation` are each parsed as one text, and a text the parser
    ///   rejects drops that script (a kept error with `"Failed to parse '<name>'!"`); with the
    ///   `molang` feature only;
    /// - a `scripts.animate` entry that names no `animations` key is noted;
    /// - an empty object in `render_controllers` or `scripts.animate` is noted (an
    ///   information) and held.
    pub fn validate(&mut self, at: &JsonPath, cx: &mut ValidateCx<'_>) {
        held_item_scale(self, at, cx.diagnostics);
        #[cfg(feature = "molang")]
        loader_scripts(self, at, cx);
        animate_names(self, at, cx.diagnostics);
        empty_conditions(self, at, cx.diagnostics);
    }
}

/// An empty object in `render_controllers` or `scripts.animate` names nothing. The
/// game uses nothing of it (the crate's own finding: vanilla's behaviour for the empty entry
/// is undefined); the entry stays where the file has it, with an information at it, and the
/// file loads.
fn empty_conditions(description: &VisualSlots, at: &JsonPath, d: &mut Diagnostics) {
    // The crate's own information: the game says nothing here.
    let mut tell = |list: &[&str], index: usize| {
        let mut entry_at = at.with(list);
        entry_at.push_index(index);
        d.push(
            Diagnostic::of(&core_codes::INVALID_VALUE)
                .severity(Severity::Info)
                .at(entry_at)
                .phase(Phase::Validate),
        );
    };
    for (index, entry) in description.render_controllers.iter().flatten().enumerate() {
        if entry.controller().is_none() {
            tell(&["render_controllers"], index);
        }
    }
    let animate = description
        .scripts
        .iter()
        .flat_map(|scripts| scripts.animate.iter().flatten());
    for (index, entry) in animate.enumerate() {
        if entry.name().is_none() {
            tell(&["scripts", "animate"], index);
        }
    }
}

/// A `held_item_scale` number at or below 0 is not used.
fn held_item_scale(slots: &VisualSlots, at: &JsonPath, d: &mut Diagnostics) {
    // The game stores the value only when it is above 0. It logs an error and loads the file:
    // an error-level finding after which the game keeps the document.
    if let Some(HeldItemScale::Number(scale)) = slots.held_item_scale
        && scale <= 0.0
    {
        d.push_kept_error(
            Diagnostic::of(codes::HELD_ITEM_SCALE_INVALID.with([FILE, &scale.to_string()]))
                .at(at.with(&["held_item_scale"]))
                .phase(Phase::Validate),
        );
    }
}

/// The rule for `initialize` and `pre_animation`: the loader's parse of the joined text.
#[cfg(feature = "molang")]
fn loader_scripts(slots: &mut VisualSlots, at: &JsonPath, cx: &mut ValidateCx<'_>) {
    use bedrock_addon_core::molang::{MolangOutcome, Side};
    let Some(check) = cx.load.molang.clone() else {
        return;
    };
    let Some(scripts) = &mut slots.scripts else {
        return;
    };
    let molang = molang_context(cx.load, cx.original_version);
    for (name, member) in [
        ("initialize", &mut scripts.initialize),
        ("pre_animation", &mut scripts.pre_animation),
    ] {
        let Some(joined) = crate::joined(member.as_deref()) else {
            continue;
        };
        let member_at = at.with(&["scripts", name]);
        let location = Location::new(None, None, member_at.clone(), None);
        let source = MolangSource::string(joined, cx.load.molang_version);
        let outcome = check.check_source(
            &source,
            &LOADER_SCRIPT,
            Side::Client,
            &molang,
            &location,
            cx.diagnostics,
        );
        if matches!(outcome, MolangOutcome::Failed(_)) {
            // "Failed to parse '{}'!" follows the parser's lines and the script is not stored.
            // The game logs it as an error and loads the file: a kept error.
            cx.diagnostics.push_kept_error(
                Diagnostic::of(codes::FAILED_TO_PARSE.with([name]))
                    .at(member_at)
                    .phase(Phase::Validate),
            );
            *member = None;
        }
    }
}

/// A `scripts.animate` entry names a key of `animations`. Vanilla lower-cases
/// the keys of `animations` and the string entries, not the keys of the object form.
fn animate_names(description: &VisualSlots, at: &JsonPath, d: &mut Diagnostics) {
    let Some(animate) = description
        .scripts
        .as_ref()
        .and_then(|s| s.animate.as_ref())
    else {
        return;
    };
    // The keys in lower case, once: every entry is looked up, not searched for.
    let names: indexmap::IndexSet<String> = description
        .animations
        .iter()
        .flat_map(IndexMap::keys)
        .map(|key| key.to_ascii_lowercase())
        .collect();
    let known = |name: &str| names.contains(name);
    for (index, entry) in animate.iter().enumerate() {
        let (name, resolved) = match entry {
            AnimateEntry::Always(name) => (name.as_str(), known(&name.to_ascii_lowercase())),
            AnimateEntry::Conditional(_) => match entry.name() {
                Some(name) => (name, known(name)),
                None => continue,
            },
        };
        if resolved {
            continue;
        }
        let mut entry_at = at.with(&["scripts", "animate"]);
        entry_at.push_index(index);
        d.push(
            Diagnostic::of(&core_codes::UNRESOLVED_REFERENCE)
                .at(entry_at)
                .phase(Phase::Validate)
                .detail(DiagnosticDetail::Reference {
                    kind: "animation",
                    id: name.to_owned(),
                    hint: None,
                }),
        );
    }
}

#[cfg(test)]
mod tests {
    use bedrock_addon_core::id::AnimationRef;
    use bedrock_addon_core::test_cost::{assert_linear_timed_at, timed};

    use super::*;
    use crate::Scripts;

    #[test]
    fn naming_the_animations_of_animate_is_linear_in_both_lists() {
        // `n` animations and as many `animate` entries, one of which names none.
        assert_linear_timed_at("visual animate names", 25_000, |n| {
            let mut slots = VisualSlots {
                animations: Some(
                    (0..n)
                        .map(|i| (format!("A{i}"), AnimationRef::parse("animation.a")))
                        .collect(),
                ),
                ..VisualSlots::default()
            };
            let mut animate: Vec<AnimateEntry> = (0..n)
                .map(|i| AnimateEntry::Always(format!("a{i}")))
                .collect();
            animate.push(AnimateEntry::Always("missing".to_owned()));
            slots.scripts = Some(Scripts {
                animate: Some(animate),
                ..Scripts::default()
            });
            let mut found = Diagnostics::default();
            let ((), took) = timed(|| animate_names(&slots, &JsonPath::new(), &mut found));
            assert_eq!(found.len(), 1);
            took
        });
    }
}
