//! The registration checks that judge one document. The checks over several documents
//! (element redefinition, pool names) run when the group combines (`lib.rs`); the references
//! are the link pass's (`link.rs`).

use super::codes;
use super::{
    BlockEntityModifier, CappedDelegate, JigsawStructure, Processor, ProcessorList, ProcessorRule,
    StructureSet, TemplatePool,
};
use bedrock_addon_core::diagnostic::{
    CodeDef, Diagnostic, Diagnostics, JsonPath, Phase, Severity, builders,
};
use bedrock_addon_core::document::{Content, JsonPlaced};
use bedrock_addon_core::load::LoadContext;

/// The texts of an identifier without a namespace.
static NO_NAMESPACE: &[&CodeDef<0>] = &[&codes::NO_NAMESPACE];
/// The texts of an empty namespace or name, or a second `:`.
static INVALID_IDENTIFIER: &[&CodeDef<0>] = &[&codes::INVALID_IDENTIFIER];
/// The texts of an illegal character in the namespace.
static NAMESPACE_INVALID: &[&CodeDef<0>] = &[&codes::ILLEGAL_CHARACTERS, &codes::NAMESPACE_INVALID];
/// The texts of an illegal character in the name.
static NAME_INVALID: &[&CodeDef<0>] = &[&codes::ILLEGAL_CHARACTERS, &codes::NAME_INVALID];

/// Why the game's identifier check (which allows the `minecraft` namespace) rejects `id`, in
/// the order it checks: `None` when it accepts it; the texts it logs
/// otherwise (none for an empty identifier). Only space, tab, LF and CR are illegal characters;
/// case, `.`, `/` and `-` pass.
pub(crate) fn identifier_failure(id: &str) -> Option<&'static [&'static CodeDef<0>]> {
    let illegal = |s: &str| s.contains([' ', '\t', '\n', '\r']);
    if id.is_empty() {
        return Some(&[]);
    }
    let Some((namespace, name)) = id.split_once(':') else {
        return Some(NO_NAMESPACE);
    };
    if namespace.is_empty() || name.is_empty() || name.contains(':') {
        return Some(INVALID_IDENTIFIER);
    }
    if illegal(namespace) {
        return Some(NAMESPACE_INVALID);
    }
    if illegal(name) {
        return Some(NAME_INVALID);
    }
    None
}

/// The findings of an identifier the check rejects: its reasons, then `register`
/// naming the identifier; each `InvalidValue` at `severity`, at `at`.
pub(crate) fn identifier_findings(
    id: &str,
    register: &'static CodeDef<1>,
    severity: Severity,
    at: &JsonPath,
) -> Option<Diagnostics> {
    let reasons = identifier_failure(id)?;
    let mut d = Diagnostics::default();
    for reason in reasons {
        d.push(
            Diagnostic::of(*reason)
                .severity(severity)
                .at(at.clone())
                .phase(Phase::Validate),
        );
    }
    d.push(
        Diagnostic::of(register.with([id]))
            .severity(severity)
            .at(at.clone())
            .phase(Phase::Validate),
    );
    Some(d)
}

/// Whether `diagnostics` are those of a document that parsed and failed a registration check
/// (a bad identifier or a bad spacing): the errors of this module are the only ones of the validation phase.
pub(crate) fn failed_registration(diagnostics: &Diagnostics) -> bool {
    diagnostics
        .iter()
        .any(|d| d.severity == Severity::Error && d.phase == Phase::Validate)
}

/// Checks the document's own identifier; an error, the document is not registered.
fn check_identifier(id: &str, register: &'static CodeDef<1>, at: &JsonPath, d: &mut Diagnostics) {
    let at = at.with(&["description", "identifier"]);
    if let Some(found) = identifier_findings(id, register, Severity::Error, &at) {
        d.extend(found);
    }
}

/// The checks of a jigsaw structure: the identifier, and an empty `start_pool`.
pub(crate) fn structure(value: &JigsawStructure, at: &JsonPath, d: &mut Diagnostics) {
    check_identifier(
        value.description.identifier.as_str(),
        &codes::STRUCTURE_REGISTER,
        at,
        d,
    );
    // The game's text for an empty `start_pool` is logged only by the builder of the
    // hard-coded structures, which the data path does not run, so the crate reports it as
    // an information of its own. The document stays loaded, as in the game.
    if value.start_pool.as_str().is_empty() {
        d.push(
            Diagnostic::of(&codes::START_POOL_EMPTY)
                .at(at.with(&["start_pool"]))
                .phase(Phase::Validate),
        );
    }
}

/// The checks of a template pool: the identifier.
pub(crate) fn pool(value: &TemplatePool, at: &JsonPath, d: &mut Diagnostics) {
    check_identifier(
        value.description.identifier.as_str(),
        &codes::POOL_REGISTER,
        at,
        d,
    );
}

/// The checks of a structure set: the identifier, each entry's identifier (an entry the
/// identifier check rejects is dropped with a kept error: the game logs it at the content
/// log's error level and goes on, the set kept) and the spacing against the separation.
pub(crate) fn set(value: &mut StructureSet, at: &JsonPath, d: &mut Diagnostics) {
    let id = value.description.identifier.as_str().to_owned();
    check_identifier(&id, &codes::SET_REGISTER, at, d);
    let mut kept = Vec::with_capacity(value.structures.len());
    let mut places = Vec::with_capacity(value.structures.len());
    for (i, entry) in std::mem::take(&mut value.structures)
        .into_iter()
        .enumerate()
    {
        let path = at.child("structures").index(i).child("structure");
        if let Some(found) = identifier_findings(
            entry.structure.as_str(),
            &codes::SET_REGISTER,
            Severity::Error,
            &path,
        ) {
            d.extend(found.into_iter().map(Diagnostic::into_kept));
        } else {
            kept.push(entry);
            places.push(i);
        }
    }
    value.structures = kept;
    // The unknown members of a dropped entry go with it; those of the others follow them.
    d.unknown_mut().left_out(&at.child("structures"), &places);
    // The spacing must be more than twice the separation.
    let placement = &value.placement;
    if u32::from(placement.spacing) <= 2 * u32::from(placement.separation) {
        d.push(
            Diagnostic::of(codes::SPACING_SEPARATION.with([id]))
                .at(at.with(&["placement"]))
                .phase(Phase::Validate),
        );
    }
}

/// The checks of a processor list: the identifier, and each rule's `append_loot` modifier (one
/// with an empty loot table is dropped with a kept error; the rule stays).
pub(crate) fn processors(value: &mut ProcessorList, at: &JsonPath, d: &mut Diagnostics) {
    check_identifier(
        value.description.identifier.as_str(),
        &codes::PROCESSORS_REGISTER,
        at,
        d,
    );
    for (i, processor) in value.processors.iter_mut().enumerate() {
        let path = at.child("processors").index(i);
        match processor {
            Processor::Rule { rules, .. } => empty_loot(rules, &path.with(&["rules"]), d),
            Processor::Capped {
                delegate: CappedDelegate::Rule { rules, .. },
                ..
            } => empty_loot(rules, &path.with(&["delegate", "rules"]), d),
            _ => {}
        }
    }
}

/// An `append_loot` modifier with an empty loot table: the game logs "Empty loot table" and leaves the modifier unset.
fn empty_loot(rules: &mut [ProcessorRule], at: &JsonPath, d: &mut Diagnostics) {
    for (i, rule) in rules.iter_mut().enumerate() {
        let empty = matches!(
            &rule.block_entity_modifier,
            Some(BlockEntityModifier::AppendLoot { loot_table, .. }) if loot_table.as_str().is_empty()
        );
        if empty {
            let path = at
                .index(i)
                .child("block_entity_modifier")
                .child("loot_table");
            // The game logs it at the content log's error level and carries on with the
            // modifier cleared: a kept error.
            d.push_kept_error(
                Diagnostic::of(&codes::EMPTY_LOOT_TABLE)
                    .at(path)
                    .phase(Phase::Validate),
            );
            rule.block_entity_modifier = None;
        }
    }
}

/// Runs the family's checks on `value` as if it were loaded from a behaviour pack with `cx`:
/// the document is written and read back, so the parse constraints and the registration checks
/// both run. The value is not changed. A value that cannot be written has no document to
/// check: that is one error finding.
pub(crate) fn validate_detached<T: JsonPlaced>(value: &T, cx: &LoadContext) -> Diagnostics {
    let path = value.place();
    match <T as Content>::to_json(value) {
        Ok(text) => match <T as Content>::parse(&path, text.as_bytes(), cx) {
            Ok(document) => document.diagnostics().clone(),
            Err(rejected) => rejected.diagnostics,
        },
        Err(error) => {
            let mut findings = Diagnostics::default();
            findings
                .push(Diagnostic::of(builders::syntax(error.to_string())).phase(Phase::Validate));
            findings
        }
    }
}
