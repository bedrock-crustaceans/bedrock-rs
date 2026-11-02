//! The entities a loot table names.

use bedrock_addon_core::diagnostic::Severity;

use super::super::{At, Findings, LinkCx, Target, kinds, unresolved};
use super::loot::{Site, walk};
use crate::families::loot::{ConditionKind, FunctionKind};

/// `set_actor_id.id` and the `entity_type` of the conditions `killed_by_entity`,
/// `entity_killed`, `damaged_by_entity` and `passenger_of_entity` name an entity. The game
/// accepts each as read: a lint.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    if !cx.lints() {
        return;
    }
    for (_, document) in cx.behavior().loot_tables() {
        walk(document, &mut |path, site| {
            let (key, entity) = match site {
                Site::Function(function) => match &function.kind {
                    FunctionKind::SetActorId { id: Some(id) } => ("id", id),
                    _ => return,
                },
                Site::Condition(condition) => match &condition.kind {
                    ConditionKind::KilledByEntity {
                        entity_type: Some(entity),
                    }
                    | ConditionKind::EntityKilled {
                        entity_type: Some(entity),
                    }
                    | ConditionKind::DamagedByEntity {
                        entity_type: Some(entity),
                    }
                    | ConditionKind::PassengerOfEntity {
                        entity_type: Some(entity),
                    } => ("entity_type", entity),
                    _ => return,
                },
                Site::Entry(_) => return,
            };
            if entity.as_str().is_empty() {
                return;
            }
            if let Target::Missing(hint) = cx.entity(entity) {
                let place = At::new(document.origin(), at![path; key], Severity::Info);
                out.push(unresolved(place, kinds::ENTITY, entity.as_str(), hint));
            }
        });
    }
}
