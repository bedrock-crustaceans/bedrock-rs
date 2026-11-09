//! A client entity draws an entity, one of a behaviour pack or one of the game's.
//! (The other half of the pairing, an entity without a client entity, is in the entity group.)

use bedrock_addon_core::diagnostic::{JsonPath, Severity};

use crate::pack::link::prelude::*;

/// Each client entity whose identifier, in lower case as the game keys it, is no entity: a
/// note at `description.identifier`. The game keeps such a definition and says nothing.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    if !cx.lints() {
        return;
    }
    for (_, document) in cx.resource().client_entities() {
        let identifier = document.value().identifier();
        let Target::Missing(hint) = cx.entity(&identifier) else {
            continue;
        };
        let mut path = JsonPath::new();
        path.push("minecraft:client_entity");
        path.push("description");
        path.push("identifier");
        let at = At::new(document.origin(), path, Severity::Info);
        out.push(unresolved(at, kinds::ENTITY, identifier.as_str(), hint));
    }
}
