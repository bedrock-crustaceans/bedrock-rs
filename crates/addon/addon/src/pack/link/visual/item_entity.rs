//! an attachable and a client entity of one
//! identifier. The game keeps both kinds in one map, so the later of the two merges into,
//! replaces or is ignored against the other; this crate keeps them apart and tells the
//! collision at the attachable.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, JsonPath, Severity};
use bedrock_addon_core::id::EntityId;

use super::super::{At, Findings, LinkCx, kinds};

/// Each attachable whose identifier, compared in lower case, is a client entity's: an
/// `IdentifierCollision` warning at its `description.identifier`.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    let entities = cx.resource().client_entities();
    if entities.is_empty() {
        return;
    }
    for (_, document) in cx.resource().attachables() {
        let attachable = document.value();
        // Both registries key their definitions by the identifier in lower case.
        let lowered = attachable.identifier();
        if entities.get(&EntityId::parse(lowered.as_str())).is_none() {
            continue;
        }
        let mut path = JsonPath::new();
        path.push("minecraft:attachable");
        path.push("description");
        path.push("identifier");
        // The shared registry map merges the two in silence: the crate reports an
        // information.
        let at = At::new(document.origin(), path, Severity::Info);
        out.push(at.reference(
            Code::from(&core_codes::IDENTIFIER_COLLISION),
            kinds::ENTITY,
            attachable.description.identifier.as_str(),
            None,
        ));
    }
}
