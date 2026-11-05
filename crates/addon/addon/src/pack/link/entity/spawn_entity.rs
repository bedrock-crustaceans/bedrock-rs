//! The entities spawn rules and spawn groups name, and the entity spawn rules
//! are for.
use bedrock_addon_core::diagnostic::{JsonPath, Severity};
use bedrock_addon_core::document::Origin;
use bedrock_addon_core::id::EntityId;

use super::super::{At, Findings, LinkCx, Target, Validator, kinds, unresolved};
use super::spawn::{self, Conditions, RULES};
use crate::families::spawn_rules::{OneOrMany, SpawnRules};

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    rules_without_entity(cx, out);
    for conditions in spawn::each_condition(cx) {
        let Conditions {
            origin,
            ref at,
            condition,
            ..
        } = conditions;
        if let Some(list) = &condition.permute_type {
            for (index, permutation) in list.as_slice().iter().enumerate() {
                let mut path = at.clone();
                path.push("minecraft:permute_type");
                // One permutation outside an array is at the member itself.
                if matches!(list, OneOrMany::Many(_)) {
                    path.push_index(index);
                }
                path.push("entity_type");
                entity(
                    cx,
                    out,
                    origin,
                    conditions.place(&path),
                    permutation.entity_type.as_ref(),
                    false,
                );
            }
        }
        if let Some(filter) = &condition.delay_filter {
            let mut path = at.clone();
            path.push("minecraft:delay_filter");
            path.push("identifier");
            let path = conditions.place(&path);
            entity(cx, out, origin, path, filter.identifier.as_ref(), true);
        }
    }
    // The 1.21.60 shape names entities in `permute_types` alone.
    for (_, document) in cx.behavior().spawn_rules() {
        let SpawnRules::Simplified(rules) = document.value() else {
            continue;
        };
        for (index, conditions) in rules.conditions.iter().flatten().enumerate() {
            for (permutation, kind) in conditions.permute_types.iter().flatten().enumerate() {
                let mut path = JsonPath::new();
                path.push(RULES);
                path.push("conditions");
                path.push_index(index);
                path.push("permute_types");
                path.push_index(permutation);
                path.push("entity_type");
                entity(
                    cx,
                    out,
                    document.origin(),
                    path,
                    kind.entity_type.as_ref(),
                    false,
                );
            }
        }
    }
}

/// Whether `name` is one of the identifiers the game's patrol spawn rules write where an
/// entity is named: no actor, let through in the rules' identifier and a delay's alone.
fn patrol(name: &EntityId) -> bool {
    crate::vanilla::names::is_spawn_rule_identifier(&super::super::canonical(name))
}

/// `name` is an entity; an absent or empty name names none (an empty delay identifier is the
/// load's finding). `identifier` says the member is a delay's identifier, where the game's
/// own files write the patrol names: the game does not look up the delay's key with a line
/// and its linked-asset check does not hold it, so a miss there is an information. A
/// permutation's entity is an error from the check.
fn entity(
    cx: &LinkCx<'_>,
    out: &mut Findings,
    origin: &Origin,
    at: JsonPath,
    name: Option<&EntityId>,
    identifier: bool,
) {
    let Some(name) = name.filter(|name| !name.as_str().is_empty()) else {
        return;
    };
    if identifier && patrol(name) {
        return;
    }
    if let Target::Missing(hint) = cx.entity(name) {
        let mut at = At::new(origin, at, Severity::Info);
        if !identifier {
            at.severity = Severity::Error;
            at = at.validator(Validator::Missing, 1, name.as_str());
        }
        out.push(unresolved(at, kinds::ENTITY, name.as_str(), hint));
    }
}

/// Spawn rules are for an entity; the game checks only that the file names one, so rules for
/// an entity nothing defines are a lint. (The other half of the pairing, a client entity
/// without an entity, is in the visual group.)
fn rules_without_entity(cx: &LinkCx<'_>, out: &mut Findings) {
    if !cx.lints() {
        return;
    }
    for (id, origin) in cx.spawn_rules_names().iter() {
        let entity: EntityId = id.clone().cast();
        if patrol(&entity) {
            continue;
        }
        if let Target::Missing(hint) = cx.entity(&entity) {
            let mut at = JsonPath::new();
            at.push(RULES);
            at.push("description");
            at.push("identifier");
            let at = At::new(origin, at, Severity::Info);
            out.push(unresolved(at, kinds::ENTITY, id.as_str(), hint));
        }
    }
}
