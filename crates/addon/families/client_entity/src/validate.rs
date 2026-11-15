//! The loader's rules on a client entity of the newest shape and the Molang walk of [`ClientEntity`]: the
//! shared members' (`visual::rules`) at the payload's `description`.

use bedrock_addon_core::molang::{MolangFields, MolangVisitor, Step};
use bedrock_addon_core::versioning::ValidateCx;

use super::{ClientEntity, PAYLOAD};

impl MolangFields for ClientEntity {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.enter(Step::Key(PAYLOAD));
        visitor.enter(Step::Key("description"));
        self.description.visit_molang(visitor);
        visitor.leave();
        visitor.leave();
    }
}

/// The loader's rules on the newest shape, after the schema and the generic Molang check
/// ([`VisualSlots::validate`](bedrock_addon_shared_visual::VisualSlots::validate)).
pub(crate) fn validate(entity: &mut ClientEntity, cx: &mut ValidateCx<'_>) {
    let at = cx.at.with(&[PAYLOAD, "description"]);
    entity.description.slots.validate(&at, cx);
}
