//! The sound event each music definition plays against the sound definitions of the stack.
//!
//! Nothing happens at load for a music definition whose `event_name` is no sound definition
//! of category `music`: the music is silent when it is played. Told as an `UnknownSound`
//! information at the definition's `event_name`, with the name as written. A stack without a
//! `sound_definitions.json` has nothing to compare with.

use bedrock_addon_core::diagnostic::{JsonPath, Severity};

use super::super::{At, Findings, LinkCx, kinds, unknown_sound};
use crate::id::SoundEventName;

/// The `event_name` of every music definition is a sound definition of category `music`.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    let definitions = cx.resource().sound_definitions();
    if definitions.layers().is_empty() {
        return;
    }
    let music = cx.resource().music();
    for (name, definition) in &music.definitions {
        if definition.event_name.is_empty() {
            continue;
        }
        let is_music = definitions
            .sound_definitions
            .get(&SoundEventName::parse(definition.event_name.as_str()))
            .is_some_and(|event| event.category.eq_ignore_ascii_case("music"));
        if is_music {
            continue;
        }
        let Some(origin) = music
            .origin(name)
            .or_else(|| music.layers().first().map(|layer| &layer.origin))
        else {
            continue;
        };
        let mut path = JsonPath::new();
        path.push(name.as_str());
        path.push("event_name");
        let at = At::new(origin, path, Severity::Info);
        out.push(unknown_sound(
            at,
            kinds::SOUND_EVENT,
            &definition.event_name,
        ));
    }
}
