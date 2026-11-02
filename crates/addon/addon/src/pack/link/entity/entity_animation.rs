//! The animations and animation controllers `description.animations` names, among those of
//! the behaviour packs. The prefix of a name picks the kind; a name with neither prefix is
//! none the pass can place, and nothing is told of it. A name nothing defines is logged as
//! "Error: can't find animation %s" at the content log's error level when the key is first
//! played, after which the entity goes on: a kept error.
use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::id::Id;

use super::super::{At, Findings, LinkCx, Target, kinds, unresolved};
use super::walk::{Entities, PAYLOAD, path};
use crate::pack::PackSide;

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings, entities: &Entities<'_>) {
    for definition in &entities.definitions {
        for (key, name) in &definition.entity.description.animations {
            let Some(Target::Missing(hint)) = cx.animation(PackSide::Behavior, &Id::parse(&**name))
            else {
                continue;
            };
            let at = path(&[PAYLOAD, "description", "animations", key]);
            let at = At::new(definition.origin, at, Severity::Error);
            out.push(unresolved(at, kinds::ANIMATION, name, hint).into_kept());
        }
    }
}
