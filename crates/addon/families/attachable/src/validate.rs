//! The loader's rules on an attachable of the newest shape and the Molang walk of [`Attachable`]: the
//! shared members' (`visual::rules`) at the payload's `description`, then `item`'s.

use bedrock_addon_core::molang::MolangFieldSpec;
use bedrock_addon_core::molang::{MolangFields, MolangVisitor, Step};
use bedrock_addon_core::versioning::ValidateCx;
pub(crate) use bedrock_addon_shared_visual::rules::{FILE, NAMED_SCRIPT, visit_controllers};

use super::read;
use super::{Attachable, ItemBinding};

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

impl MolangFields for Attachable {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.enter(Step::Key(read::PAYLOAD));
        visitor.enter(Step::Key("description"));
        self.description.slots.visit_molang(visitor);
        visit_item(&mut self.description.item, visitor);
        visitor.leave();
        visitor.leave();
    }
}

/// The loader's rules on the newest shape, after the schema and the generic Molang check
/// ([`VisualSlots::validate`](bedrock_addon_shared_visual::VisualSlots::validate)).
pub(crate) fn validate(attachable: &mut Attachable, cx: &mut ValidateCx<'_>) {
    let at = cx.at.with(&[read::PAYLOAD, "description"]);
    attachable.description.slots.validate(&at, cx);
}
