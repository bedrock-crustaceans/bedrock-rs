//! The loader's rules on an attachable of the newest shape, the Molang field specs and the
//! Molang walk of [`Attachable`].

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Phase,
};
#[cfg(feature = "molang")]
use bedrock_addon_core::diagnostic::{Location, Severity};
use bedrock_addon_core::json::StrictString;
#[cfg(feature = "molang")]
use bedrock_addon_core::molang::MolangSource;
use bedrock_addon_core::molang::{
    EmptyRule, Molang, MolangFailure, MolangFieldSpec, MolangFields, MolangVisitor, ReaderKind,
    Step,
};
use bedrock_addon_core::versioning::{UpgradeCx, ValidateCx};
use bedrock_addon_core::visual::{AnimateEntry, ControllerUse, HeldItemScale, Scripts};
use indexmap::{IndexMap, IndexSet};

use super::read::{self, text};
use super::{Attachable, ItemBinding};

/// A named script as the schema types it (an expression): a number, a boolean or a string
/// at the pack's version. A string the parser rejects fails the schema, so the document.
// The 1.8 and 1.10 typed-node members: a rejection is a schema failure and the file is
// skipped.
pub(crate) const NAMED_SCRIPT: MolangFieldSpec = MolangFieldSpec {
    reader: ReaderKind::SchemaValidated,
    on_failure: MolangFailure::RejectDocument,
    ..MolangFieldSpec::DEFAULT
};

/// The condition of an object-form `render_controllers` or `scripts.animate` entry: typed as
/// a named script is.
pub(crate) const CONDITION: MolangFieldSpec = NAMED_SCRIPT;

/// The condition of an object-form `item`: typed as a named script is, so a text the parser
/// rejects fails the schema and the document. Every member is
/// validated, not only the one that binds.
pub(crate) const ITEM_CONDITION: MolangFieldSpec = NAMED_SCRIPT;

/// The conditions of an object-form `item`, a field per member.
pub(crate) fn visit_item(item: &mut Option<ItemBinding>, visitor: &mut MolangVisitor<'_>) {
    if let Some(ItemBinding::Conditional(members)) = item {
        visitor.field("item", &ITEM_CONDITION, members);
    }
}

/// `held_item_scale` in its Molang form (engine 1.26.30 and up): compiled with the pack's
/// version. The current schema types it as an expression, which fails validation when the
/// parser rejects the text, and the loader skips a file that fails its schema: the parser's
/// lines are errors and the document is refused. Validation fails an array as well; the typed
/// field (`HeldItemScale`, a number or Molang) refuses one before this spec is asked, so the
/// reader's array form is never used.
const HELD_ITEM_SCALE: MolangFieldSpec = MolangFieldSpec {
    reader: ReaderKind::ScalarOrArray,
    empty: EmptyRule::Compile,
    on_failure: MolangFailure::RejectDocument,
    ..MolangFieldSpec::DEFAULT
};

/// `initialize` / `pre_animation` as the loader parses them: the strings joined into one
/// text. A rejected text drops that script; the file loads.
#[cfg(feature = "molang")]
const LOADER_SCRIPT: MolangFieldSpec = MolangFieldSpec {
    reader: ReaderKind::SchemaValidated,
    on_failure: MolangFailure::DropEntry,
    // The parser's lines come before "Failed to parse" and are content-log errors; the
    // definition loads on without the script, so they are kept errors.
    error_severity: Severity::Error,
    error_kept: true,
    ..MolangFieldSpec::DEFAULT
};

/// Vanilla's text for a named script the loader cannot parse; `{}` is the member name.
#[cfg(feature = "molang")]
pub(crate) const FAILED_TO_PARSE: &str = "Failed to parse '{}'!";
/// Vanilla's text for a `held_item_scale` at or below 0: the file, then the value.
pub(crate) const HELD_ITEM_SCALE_TEXT: &str = "In file '{}', parameter \"held_item_scale\" has invalid value: must be greater than zero, was '{}'.";
/// Stands for the file's path in a text until the load knows it.
pub(crate) const FILE: &str = "\u{0}file";

/// The text the loader builds for `initialize` or `pre_animation`: the strings joined with no
/// separator. `None` when the member is absent or the text is empty.
pub(crate) fn joined(parts: Option<&[StrictString]>) -> Option<String> {
    // An array is read as its string elements concatenated with no separator.
    let text: String = parts?.iter().map(|part| part.0.as_str()).collect();
    (!text.is_empty()).then_some(text)
}

/// The conditional entries of `render_controllers`, each a field per member.
pub(crate) fn visit_controllers(
    controllers: Option<&mut Vec<ControllerUse>>,
    visitor: &mut MolangVisitor<'_>,
) {
    let Some(controllers) = controllers else {
        return;
    };
    visitor.enter(Step::Key("render_controllers"));
    for (index, controller) in controllers.iter_mut().enumerate() {
        if let ControllerUse::Conditional(members) = controller {
            visitor.field(index, &CONDITION, members);
        }
    }
    visitor.leave();
}

/// The conditional entries of `scripts.animate`, each a field per member.
pub(crate) fn visit_animate(
    animate: Option<&mut Vec<AnimateEntry>>,
    spec: &MolangFieldSpec,
    visitor: &mut MolangVisitor<'_>,
) {
    let Some(animate) = animate else { return };
    visitor.enter(Step::Key("animate"));
    for (index, entry) in animate.iter_mut().enumerate() {
        if let AnimateEntry::Conditional(members) = entry {
            visitor.field::<IndexMap<String, Molang>>(index, spec, members);
        }
    }
    visitor.leave();
}

impl MolangFields for Attachable {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let d = &mut self.description;
        visitor.enter(Step::Key(read::PAYLOAD));
        visitor.enter(Step::Key("description"));
        // The value is built without a version; the walk gives it the pack's here.
        if let Some(HeldItemScale::Molang(scale)) = &mut d.held_item_scale {
            visitor.field("held_item_scale", &HELD_ITEM_SCALE, scale);
        }
        visit_controllers(d.render_controllers.as_mut(), visitor);
        if let Some(scripts) = &mut d.scripts {
            visitor.enter(Step::Key("scripts"));
            visit_named(scripts, visitor);
            visit_animate(scripts.animate.as_mut(), &CONDITION, visitor);
            visitor.leave();
        }
        visit_item(&mut d.item, visitor);
        visitor.leave();
        visitor.leave();
    }
}

/// The named scripts the schema types as Molang.
fn visit_named(scripts: &mut Scripts, visitor: &mut MolangVisitor<'_>) {
    visitor.field("scale", &NAMED_SCRIPT, &mut scripts.scale);
    visitor.field("scaleX", &NAMED_SCRIPT, &mut scripts.scale_x);
    visitor.field("scalex", &NAMED_SCRIPT, &mut scripts.scalex);
    visitor.field("scaleY", &NAMED_SCRIPT, &mut scripts.scale_y);
    visitor.field("scaley", &NAMED_SCRIPT, &mut scripts.scaley);
    visitor.field("scaleZ", &NAMED_SCRIPT, &mut scripts.scale_z);
    visitor.field("scalez", &NAMED_SCRIPT, &mut scripts.scalez);
    visitor.field("parent_setup", &NAMED_SCRIPT, &mut scripts.parent_setup);
    visitor.field(
        "should_update_bones_and_effects_offscreen",
        &NAMED_SCRIPT,
        &mut scripts.should_update_bones_and_effects_offscreen,
    );
    visitor.field(
        "should_update_effects_offscreen",
        &NAMED_SCRIPT,
        &mut scripts.should_update_effects_offscreen,
    );
    visitor.field(
        "hide_held_items",
        &NAMED_SCRIPT,
        &mut scripts.hide_held_items,
    );
}

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

/// The loader's rules on the newest shape (`_loadActorResourceDefinition`), after the schema
/// and the generic Molang check:
/// - `held_item_scale` as a number at or below 0 is not used (a kept error with vanilla's text);
/// - `initialize` and `pre_animation` are each parsed as one text, and a text the parser
///   rejects drops that script (a kept error with "Failed to parse '<name>'!"); with the `molang`
///   feature only;
/// - a `scripts.animate` entry that names no `animations` key is noted;
/// - an empty object in `render_controllers` or `scripts.animate` is noted (an information) and held.
pub(crate) fn validate(attachable: &mut Attachable, cx: &mut ValidateCx<'_>) {
    let at = cx.at.with(&[read::PAYLOAD, "description"]);
    held_item_scale(attachable, &at, cx.diagnostics);
    #[cfg(feature = "molang")]
    loader_scripts(attachable, &at, cx);
    animate_names(attachable, &at, cx.diagnostics);
    empty_conditions(attachable, &at, cx.diagnostics);
}

/// An empty object in `render_controllers` or `scripts.animate` names nothing. The game
/// uses nothing of it (vanilla reads the first member of the empty map, which is undefined
/// behaviour; the information is the crate's own); the
/// entry stays where the file has it, with an information at it, and the file loads.
fn empty_conditions(attachable: &Attachable, at: &JsonPath, d: &mut Diagnostics) {
    // Entry-level (entry skipped, file loads); the crate reports an information.
    let mut tell = |list: &[&str], index: usize| {
        let mut entry_at = at.with(list);
        entry_at.push_index(index);
        d.push(
            Diagnostic::info(DiagnosticCode::InvalidValue { text: None })
                .at(entry_at)
                .phase(Phase::Validate),
        );
    };
    let description = &attachable.description;
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

/// A `held_item_scale` number at or below 0.
fn held_item_scale(attachable: &Attachable, at: &JsonPath, d: &mut Diagnostics) {
    // The game stores the value only when it is above 0. It logs an error and loads the
    // file: a kept error.
    if let Some(HeldItemScale::Number(scale)) = attachable.description.held_item_scale
        && scale <= 0.0
    {
        d.push_kept_error(
            Diagnostic::error(DiagnosticCode::InvalidValue {
                text: text(HELD_ITEM_SCALE_TEXT, &[FILE, &scale.to_string()]),
            })
            .at(at.with(&["held_item_scale"]))
            .phase(Phase::Validate),
        );
    }
}

/// The loader's parse of the joined text of `initialize` and `pre_animation`.
#[cfg(feature = "molang")]
fn loader_scripts(attachable: &mut Attachable, at: &JsonPath, cx: &mut ValidateCx<'_>) {
    use bedrock_addon_core::molang::{MolangOutcome, Side};
    let Some(check) = cx.load.molang.clone() else {
        return;
    };
    let Some(scripts) = &mut attachable.description.scripts else {
        return;
    };
    let molang = molang_context(cx.load, cx.original_version);
    for (name, member) in [
        ("initialize", &mut scripts.initialize),
        ("pre_animation", &mut scripts.pre_animation),
    ] {
        let Some(joined) = joined(member.as_deref()) else {
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
            // "Failed to parse '{}'!" comes after the parser's lines and the script is not
            // stored. The game logs it as an error and loads the file: a kept error.
            cx.diagnostics.push_kept_error(
                Diagnostic::error(DiagnosticCode::DroppedField {
                    text: text(FAILED_TO_PARSE, &[name]),
                })
                .at(member_at)
                .phase(Phase::Validate),
            );
            *member = None;
        }
    }
}

/// A `scripts.animate` entry names a key of `animations`. Vanilla lower-cases
/// the keys of `animations` and the string entries, not the keys of the object form.
fn animate_names(attachable: &Attachable, at: &JsonPath, d: &mut Diagnostics) {
    let description = &attachable.description;
    let Some(animate) = description
        .scripts
        .as_ref()
        .and_then(|s| s.animate.as_ref())
    else {
        return;
    };
    // The keys in lower case, once: every entry is looked up, not searched for.
    let names: IndexSet<String> = description
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
            Diagnostic::info(DiagnosticCode::UnresolvedReference { text: None })
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
    use bedrock_addon_core::test_cost::{assert_linear_timed_at, timed};

    use bedrock_addon_core::id::{AnimationRef, AttachableId};

    use super::*;

    #[test]
    fn naming_the_animations_of_animate_is_linear_in_both_lists() {
        // `n` animations and as many `animate` entries, one of which names none.
        assert_linear_timed_at("attachable animate names", 25_000, |n| {
            let mut attachable = Attachable::new(AttachableId::parse("demo:a"));
            attachable.description.animations = Some(
                (0..n)
                    .map(|i| (format!("A{i}"), AnimationRef::parse("animation.a")))
                    .collect(),
            );
            let mut animate: Vec<AnimateEntry> = (0..n)
                .map(|i| AnimateEntry::Always(format!("a{i}")))
                .collect();
            animate.push(AnimateEntry::Always("missing".to_owned()));
            attachable.description.scripts = Some(Scripts {
                animate: Some(animate),
                ..Scripts::default()
            });
            let mut found = Diagnostics::default();
            let ((), took) = timed(|| animate_names(&attachable, &JsonPath::new(), &mut found));
            assert_eq!(found.len(), 1);
            took
        });
    }
}
