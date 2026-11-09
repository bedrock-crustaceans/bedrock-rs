//! The entities an item places or shoots (`entity_placer.entity`,
//! `projectile.projectile_entity`): both are event-identifier references, so an `<event>` may
//! follow the name.

use bedrock_addon_core::diagnostic::{JsonPath, Severity};
use bedrock_addon_core::id::EntityId;

use crate::pack::link::prelude::*;
use super::item::{Entry, component, items};
use crate::families::item::component::{EntityPlacer, Projectile};

/// `minecraft:entity_placer.entity` and `minecraft:projectile.projectile_entity` name
/// an entity of the stack or of the game; an `<event>` after the name is not part of it.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    for item in items(cx) {
        if let Some(placer) = item.components.get::<EntityPlacer>()
            && let Some(entity) = &placer.entity
        {
            let path = at![&component("minecraft:entity_placer"); "entity"];
            resolve(cx, out, &item, path, entity);
        }
        if let Some(projectile) = item.components.get::<Projectile>() {
            let path = at![&component("minecraft:projectile"); "projectile_entity"];
            resolve(cx, out, &item, path, &projectile.projectile_entity);
        }
    }
}

/// One entity name of `item` at `path`.
fn resolve(cx: &LinkCx<'_>, out: &mut Findings, item: &Entry<'_>, path: JsonPath, name: &EntityId) {
    if name.as_str().is_empty() {
        return;
    }
    if let Target::Missing(hint) = cx.entity(name) {
        let bare = name.as_str().split('<').next().unwrap_or_default();
        // The linked-asset check's grading; the severity here is read by nothing else.
        let place = At::new(item.origin, path, Severity::Error).validator(Validator::Missing, 1, bare);
        out.push(unresolved(place, kinds::ENTITY, name.as_str(), hint));
    }
}
