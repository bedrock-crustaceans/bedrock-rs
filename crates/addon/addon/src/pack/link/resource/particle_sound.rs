//! The sound an event of a particle effect plays is a level sound event of the game.
//!
//! The game resolves the name when it reads the effect and raises a content error for one it
//! does not know; the effect stays. Told here as an error at `event_name`, with the name as
//! written. The names of the game's events are lower-case and the name is compared as it is
//! written: no case is folded.

use bedrock_addon_core::diagnostic::Severity;

use super::super::{At, Findings, LinkCx, kinds, unknown_sound};
use super::particle::each_event_node;

/// `sound_effect.event_name` of every event node is a level sound event: one the host
/// says the game has, or without a host one the crate knows. One that is none is an
/// `UnknownSound` error at the member. An empty name is no reference: the load has told the
/// member it stands for.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    for (_, document) in cx.resource().particles() {
        each_event_node(document, |at, node| {
            let Some(sound) = &node.sound_effect else {
                return;
            };
            let name = sound.event_name.as_str();
            if name.is_empty() || !cx.level_sound_event(name).is_missing() {
                return;
            }
            let mut at = at.clone();
            at.push("sound_effect");
            at.push("event_name");
            // The game logs "Event name '%s' is not a valid LevelSoundEvent" at the error
            // level and uses the undefined event; the effect keeps the event: a kept error.
            // The game lower-cases the name first, while the crate compares it as written (a
            // known gap).
            let at = At::new(document.origin(), at, Severity::Error);
            out.push(unknown_sound(at, kinds::LEVEL_SOUND_EVENT, name).into_kept());
        });
    }
}
