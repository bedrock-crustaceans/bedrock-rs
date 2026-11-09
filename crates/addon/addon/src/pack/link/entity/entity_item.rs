//! The items an entity's components name. A name nothing defines reaches the game's deferred
//! descriptor resolution, which writes "Unknown item during Deferred `ItemDescriptor`
//! resolution: %s" at the lowest log level only: an information.
use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::id::ItemId;
use bedrock_addon_shared_shapes::{ItemDescriptor, ItemDescriptorForm};

use super::walk::Entities;
use crate::families::entity::links::{Reference, ReferenceKind};
use crate::pack::link::prelude::*;

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings, entities: &Entities<'_>) {
    entities.each(ReferenceKind::Item, &mut |definition, at, found| {
        let Reference::Item {
            descriptor,
            or_item_tag,
        } = &found.reference
        else {
            return;
        };
        let ItemDescriptor::Item { id, form, .. } = descriptor else {
            // An item tag (the shared resolver's lint) or a tag expression (the game's).
            let member = Member {
                origin: definition.origin,
                path: &at,
                severity: Severity::Info,
                validator: None,
                states: States::Lint,
            };
            item_descriptor(cx, out, descriptor, member);
            return;
        };
        // The member's own normalisation: the legacy `name:aux` of an entity file
        // (`water_bucket:0`) names the item `name`.
        let looked_up = without_aux(id.as_str()).map_or_else(|| id.clone(), ItemId::parse);
        let Target::Missing(hint) = cx.item(&looked_up) else {
            return;
        };
        // A name the game's own entity files write here and the game does not have: let
        // through in these rows alone.
        if crate::vanilla::names::is_entity_file_only_item(&canonical(&looked_up)) {
            return;
        }
        if *or_item_tag {
            // An item or an item tag (`shareables.items[].item`): without a host the game's
            // tags are not known, so a name that is neither here is a lint.
            if !cx.item_tag(id.as_str()).is_missing() || !cx.lints() {
                return;
            }
        }
        let mut at = at.clone();
        match form {
            ItemDescriptorForm::Item => at.push("item"),
            ItemDescriptorForm::Name => at.push("name"),
            _ => {}
        }
        let at = At::new(definition.origin, at, Severity::Info);
        out.push(unresolved(at, kinds::ITEM, id.as_str(), hint));
    });
}

/// The name of a legacy `name:aux` item text without a namespace (`bucket:0` → `bucket`):
/// one `:` followed by digits alone. `None` for any other text (`ns:name`, `ns:name:aux`,
/// `name`), which the item lookup reads itself.
fn without_aux(text: &str) -> Option<&str> {
    let (name, aux) = text.split_once(':')?;
    (!name.is_empty() && !aux.is_empty() && aux.bytes().all(|byte| byte.is_ascii_digit()))
        .then_some(name)
}
