//! The Molang members of the goals: `eat_block.success_chance`, the `tags` expressions of
//! item and block descriptors, and the legacy `condition` of a trigger.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::molang::{
    self, Admission, Molang, MolangFailure, MolangFieldSpec, MolangVersion, MolangVersionSource,
    MolangVisitor, RawVersion, ReadField, ReaderKind, Step,
};
use bedrock_addon_core::shared::{BlockDescriptor, EventTrigger, ItemDescriptor};

use super::common::LegacyConditions;

/// `eat_block.success_chance`: a cereal expression node, its string form at the pack's Molang
/// version; a rejected expression is an error and is kept.
pub(crate) const SUCCESS_CHANCE: MolangFieldSpec = MolangFieldSpec::DEFAULT;

/// `eat_block.success_chance` read by the lenient conversion of an older document: the
/// schema-validated reader, and a rejected expression is a kept error that gives the member's
/// default (0.02).
pub(crate) const SUCCESS_CHANCE_CONVERTED: MolangFieldSpec = MolangFieldSpec {
    reader: ReaderKind::SchemaValidated,
    on_failure: MolangFailure::ReplaceWith(0.02),
    // The game logs a rejected expression at the error level and still loads the entity: the
    // expression node stores the text and the parse result is dropped, so the finding is a kept
    // error.
    error_severity: Severity::Error,
    error_kept: true,
    ..MolangFieldSpec::DEFAULT
};

/// A legacy trigger `condition`: the lenient reader, the pack's version, and a rejected
/// expression a kept error (the typed trigger has no condition).
const CONDITION: MolangFieldSpec = MolangFieldSpec {
    reader: ReaderKind::LenientVersioned,
    // The game binds `condition` as an expression node and drops the parse result; it logs a
    // rejected expression at the error level and keeps the document: a kept error.
    error_severity: Severity::Error,
    error_kept: true,
    ..MolangFieldSpec::DEFAULT
};

/// Adds the trigger member `key`, when present, to `triggers`.
pub(crate) fn push_trigger<'t>(
    triggers: &mut Vec<(String, &'t EventTrigger)>,
    key: &str,
    trigger: Option<&'t EventTrigger>,
) {
    if let Some(trigger) = trigger {
        triggers.push((key.to_owned(), trigger));
    }
}

/// Adds each trigger of the list member `key` to `triggers`, at `key/<index>`.
pub(crate) fn push_triggers<'t>(
    triggers: &mut Vec<(String, &'t EventTrigger)>,
    key: &str,
    list: Option<&'t [EventTrigger]>,
) {
    for (i, trigger) in list.into_iter().flatten().enumerate() {
        triggers.push((format!("{key}/{i}"), trigger));
    }
}

/// Visits the legacy `condition` of each trigger (kept in the trigger's `extra`), read once
/// into `cache`, where pass 1 settles its version and pass 2 finds it.
pub(crate) fn visit_conditions(
    visitor: &mut MolangVisitor<'_>,
    cache: &mut LegacyConditions,
    triggers: &[(String, &EventTrigger)],
) {
    let entries = cache.entries.get_or_insert_with(|| {
        triggers
            .iter()
            .filter_map(|(path, trigger)| {
                let value = trigger.extra.get("condition")?;
                // A value the lenient reader refuses (an array, a bare object) is not a Molang
                // member; it stays in `extra`, reported as an unknown member.
                match molang::read(value, ReaderKind::LenientVersioned, RawVersion(-1)) {
                    Ok(ReadField::Value(value)) => Some((path.clone(), value)),
                    _ => None,
                }
            })
            .collect::<Vec<(String, Molang)>>()
    });
    for (path, value) in entries.iter_mut() {
        let steps: Vec<&str> = path.split('/').collect();
        for step in &steps {
            match step.parse::<usize>() {
                Ok(index) => visitor.enter(Step::from(index)),
                Err(_) => visitor.enter(Step::from(*step)),
            }
        }
        visitor.field("condition", &CONDITION, value);
        for _ in &steps {
            visitor.leave();
        }
    }
}

/// A descriptor's `tags`: a string only, at the cereal descriptors' constant version 13, with the
/// `tags` query set. A rejected expression is a kept error and the descriptor is kept.
// The game parses a descriptor's `tags` expression at the constant Molang version 13, and a
// rejected expression does not stop the entity from loading.
const TAGS: MolangFieldSpec = MolangFieldSpec {
    version: MolangVersionSource::Constant(MolangVersion::V13),
    admission: Admission::Tags,
    // The game logs a rejected `tags` expression at the error level and keeps the descriptor
    // (it is returned even when the parse fails): a kept error.
    error_severity: Severity::Error,
    error_kept: true,
    ..MolangFieldSpec::DEFAULT
};

/// Visits the `tags` expression of each item descriptor of list member `key`.
pub(crate) fn visit_items(
    visitor: &mut MolangVisitor<'_>,
    key: &str,
    items: &mut Option<Vec<ItemDescriptor>>,
) {
    let Some(items) = items else { return };
    visitor.enter(Step::from(key));
    for (i, item) in items.iter_mut().enumerate() {
        if let ItemDescriptor::Tags { tags, .. } = item {
            visitor.enter(Step::from(i));
            visitor.field("tags", &TAGS, tags);
            visitor.leave();
        }
    }
    visitor.leave();
}

/// Visits the `tags` expression of each block descriptor of list member `key`.
pub(crate) fn visit_blocks(
    visitor: &mut MolangVisitor<'_>,
    key: &str,
    blocks: &mut Option<Vec<BlockDescriptor>>,
) {
    let Some(blocks) = blocks else { return };
    visitor.enter(Step::from(key));
    for (i, block) in blocks.iter_mut().enumerate() {
        visitor.enter(Step::from(i));
        visit_block(visitor, block);
        visitor.leave();
    }
    visitor.leave();
}

/// Visits the `tags` expression of one block descriptor, at the visitor's current step.
pub(crate) fn visit_block(visitor: &mut MolangVisitor<'_>, block: &mut BlockDescriptor) {
    if let BlockDescriptor::Tags { tags, .. } | BlockDescriptor::Mixed { tags, .. } = block {
        visitor.field("tags", &TAGS, tags);
    }
}
