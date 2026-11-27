//! The sounds the bindings of `sounds.json` name against the sound definitions of the stack.
//!
//! Nothing happens at load for a binding of a sound no definition names: the sound is silent
//! when it is played. Told as an `UnknownSound` information at the binding, with the name as
//! written. A stack without a `sound_definitions.json` has nothing to compare with.

use bedrock_addon_core::diagnostic::{JsonPath, Severity};
use bedrock_addon_core::document::{Merged, RawLayer};

use crate::families::sound_definitions::SoundDefinitions;
use crate::families::sounds_json::SoundBindings;
use crate::id::SoundEventName;
use crate::pack::link::prelude::*;
use bedrock_addon_core::load::sound::path_member;

/// Every sound a binding names is a sound definition of the merged `sound_definitions.json`.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    let definitions: &Merged<SoundDefinitions, RawLayer> = cx.resource().sound_definitions();
    if definitions.layers().is_empty() {
        return;
    }
    let bindings = cx.resource().sounds_json();
    let top = bindings.layers().first().map(|layer| &layer.origin);
    for (at, sound) in bound_sounds(bindings.value()) {
        let defined = definitions
            .sound_definitions
            .get(&SoundEventName::parse(sound.as_str()))
            .is_some();
        if sound.is_empty() || defined {
            continue;
        }
        let origin = path_member(&at, 0)
            .and_then(|section| bindings.origin(&section))
            .or(top);
        let Some(origin) = origin else {
            continue;
        };
        let at = At::new(origin, at, Severity::Info);
        out.push(unknown_sound(at, kinds::SOUND_EVENT, &sound));
    }
}

/// One bound sound: the path `section/key` and the name.
fn collect(section: &[&str], key: &str, sound: &str) -> (JsonPath, String) {
    let mut at = JsonPath::new();
    for part in section {
        at.push(part);
    }
    at.push(key);
    (at, sound.to_owned())
}

/// The sounds of `interactive_sounds.entity_sounds`.
fn interactive_sounds(value: &SoundBindings) -> Vec<(JsonPath, String)> {
    let mut owned = Vec::new();
    let interactive = &value.interactive_sounds.entity_sounds;
    for (event, surfaces) in &interactive.defaults.events {
        for (surface, sound) in surfaces {
            owned.push(collect(
                &[
                    "interactive_sounds",
                    "entity_sounds",
                    "defaults",
                    "events",
                    event,
                ],
                surface,
                &sound.sound,
            ));
        }
    }
    for (entity, entry) in &interactive.entities {
        for (event, surfaces) in &entry.sounds.events {
            for (surface, sound) in surfaces {
                owned.push(collect(
                    &[
                        "interactive_sounds",
                        "entity_sounds",
                        "entities",
                        entity,
                        "events",
                        event,
                    ],
                    surface,
                    &sound.sound,
                ));
            }
        }
    }
    owned
}

/// Every sound a binding names, with the path of the binding.
fn bound_sounds(value: &SoundBindings) -> Vec<(JsonPath, String)> {
    let mut owned: Vec<(JsonPath, String)> = Vec::new();
    for (event, sound) in &value.individual_event_sounds.events {
        owned.push(collect(
            &["individual_event_sounds", "events"],
            event,
            &sound.sound,
        ));
    }
    for (name, sound) in &value.individual_named_sounds.sounds {
        owned.push(collect(
            &["individual_named_sounds", "sounds"],
            name,
            &sound.sound,
        ));
    }
    for (section, blocks) in [
        (&["block_sounds"][..], &value.block_sounds),
        (
            &["interactive_sounds", "block_sounds"][..],
            &value.interactive_sounds.block_sounds,
        ),
    ] {
        for (block, table) in blocks {
            for (event, sound) in &table.sounds.events {
                let mut path: Vec<&str> = section.to_vec();
                path.extend([block.as_str(), "events"]);
                owned.push(collect(&path, event, &sound.sound));
            }
        }
    }
    for (event, sound) in &value.entity_sounds.defaults.events {
        owned.push(collect(
            &["entity_sounds", "defaults", "events"],
            event,
            &sound.sound,
        ));
    }
    for (entity, entry) in &value.entity_sounds.entities {
        for (event, sound) in &entry.sounds.events {
            owned.push(collect(
                &["entity_sounds", "entities", entity, "events"],
                event,
                &sound.sound,
            ));
        }
        for (variant, table) in entry.variants.iter().flat_map(|variants| &variants.map) {
            for (event, sound) in &table.events {
                owned.push(collect(
                    &[
                        "entity_sounds",
                        "entities",
                        entity,
                        "variants",
                        "map",
                        variant,
                        "events",
                    ],
                    event,
                    &sound.sound,
                ));
            }
        }
    }
    owned.extend(interactive_sounds(value));
    owned
}
