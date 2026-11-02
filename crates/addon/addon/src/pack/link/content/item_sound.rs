//! The sounds an item names. The game reads the member as a sound-event name: a string that
//! names a level sound event resolves to it; any other string is kept and checked against the
//! resource-pack sound definitions.

use bedrock_addon_core::diagnostic::{JsonPath, Severity};
use bedrock_addon_core::id::SoundEventName;

use super::super::{At, Findings, LinkCx, kinds, unknown_sound};
use super::item::{Entry, component, items};
use crate::families::item::component::{DurabilitySensor, Record, SwingSounds, UseModifiers};

/// A sound an item names (`record.sound_event`, each
/// `durability_sensor.durability_thresholds[].sound_event`, the three `swing_sounds` and
/// `use_modifiers.start_sound`) is a level sound event of the game, compared without case, or
/// a sound of the resource stack's `sound_definitions.json`, compared as written. A name that
/// is neither is an information: the game answers without a line, and no loader of these
/// members logs one.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    for item in items(cx) {
        let components = item.components;
        if let Some(record) = components.get::<Record>() {
            let path = at![&component("minecraft:record"); "sound_event"];
            sound(cx, out, &item, path, record.sound_event.as_ref());
        }
        if let Some(sensor) = components.get::<DurabilitySensor>() {
            for (index, threshold) in sensor.durability_thresholds.iter().enumerate() {
                let path = at![&component("minecraft:durability_sensor"); "durability_thresholds", index, "sound_event"];
                sound(cx, out, &item, path, threshold.sound_event.as_ref());
            }
        }
        if let Some(swing) = components.get::<SwingSounds>() {
            for (key, name) in [
                ("attack_miss", &swing.attack_miss),
                ("attack_hit", &swing.attack_hit),
                ("attack_critical_hit", &swing.attack_critical_hit),
            ] {
                let path = at![&component("minecraft:swing_sounds"); key];
                sound(cx, out, &item, path, name.as_ref());
            }
        }
        if let Some(modifiers) = components.get::<UseModifiers>() {
            let path = at![&component("minecraft:use_modifiers"); "start_sound"];
            sound(cx, out, &item, path, modifiers.start_sound.as_ref());
        }
    }
}

/// One sound name of `item` at `path`.
fn sound(
    cx: &LinkCx<'_>,
    out: &mut Findings,
    item: &Entry<'_>,
    path: JsonPath,
    name: Option<&SoundEventName>,
) {
    let Some(name) = name.filter(|name| !name.as_str().is_empty()) else {
        return;
    };
    let event = name.as_str().to_ascii_lowercase();
    if cx.level_sound_event(&event).is_missing() && cx.sound_definition(name).is_missing() {
        let place = At::new(item.origin, path, Severity::Info);
        out.push(unknown_sound(place, kinds::SOUND_EVENT, name.as_str()));
    }
}
