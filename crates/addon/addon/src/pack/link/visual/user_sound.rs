//! `description.sound_effects` of a client entity or an atomic client entity: each value names
//! a sound event, used as written. The game checks nothing when it loads: a note.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::id::SoundEventName;

use super::super::{At, Findings, LinkCx, kinds, unknown_sound};
use super::user::User;

/// `sound_effects`: each value names a sound event of `sound_definitions.json`, as written. A
/// miss is an `UnknownSound` finding.
pub(super) fn check(cx: &LinkCx<'_>, out: &mut Findings, user: &User<'_>) {
    if !cx.lints() {
        return;
    }
    for (key, value) in user.sound_effects.into_iter().flatten() {
        let name = SoundEventName::parse(value.as_str());
        if cx.sound_definition(&name).is_missing() {
            let at = At::new(
                user.origin,
                user.entry("sound_effects", key),
                Severity::Info,
            );
            out.push(unknown_sound(at, kinds::SOUND_EVENT, value.as_str()));
        }
    }
}
