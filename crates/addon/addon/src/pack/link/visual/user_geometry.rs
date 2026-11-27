//! `description.geometry` of a client entity, an attachable or an atomic client entity: each
//! value names a geometry. The game resolves it when it draws and logs nothing: a note.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::id::GeometryId;

use super::user::User;
use crate::pack::link::prelude::*;

/// `geometry`: each value names a geometry.
pub(super) fn check(cx: &LinkCx<'_>, out: &mut Findings, user: &User<'_>) {
    if !cx.lints() {
        return;
    }
    for (key, value) in user.geometry.into_iter().flatten() {
        let name = user.folded(value.as_str());
        if let Target::Missing(hint) = cx.geometry(&GeometryId::parse(&*name)) {
            let at = At::new(user.origin, user.entry("geometry", key), Severity::Info);
            out.push(unresolved(at, kinds::GEOMETRY, &name, hint));
        }
    }
}
