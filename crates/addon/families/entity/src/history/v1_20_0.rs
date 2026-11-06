//! Format 1.20.0: the struct and the step to 1.20.60.

use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};
use facet_value::{VArray, VObject};

use super::Payload;
use super::ops::{Told, section_b, sections};
use bedrock_addon_core::diagnostic::JsonPath;

/// An entity document as written in format 1.20.0 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 20, 0>;

/// 1.20.0 → 1.20.60 (gate 1.20.60).
impl LegacyStep<1, 20, 0> for Payload {
    type Next = super::v1_20_60::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        if section_b(&payload, cx) {
            let mut told = Told::default();
            for section in sections(&mut payload) {
                let at = section.at.child("minecraft:entity_sensor");
                if let Some(sensor) = section.members.object("minecraft:entity_sensor") {
                    subsensors(sensor, &at, &mut told);
                }
            }
            told.report(cx);
        }
        Legacy(payload)
    }
}

/// B, `entity_sensor`: the single sensor's root members move into `subsensors[0]`
/// (`sensor_range` as `range`); `relative_range` and `find_players_only` stay at the root.
fn subsensors(sensor: &mut VObject, at: &JsonPath, told: &mut Told) {
    const MOVED: [(&str, &str); 6] = [
        ("event", "event"),
        ("sensor_range", "range"),
        ("require_all", "require_all"),
        ("minimum_count", "minimum_count"),
        ("maximum_count", "maximum_count"),
        ("event_filters", "event_filters"),
    ];
    let mut subsensor = VObject::new();
    let mut moved = Vec::new();
    for (old, new) in MOVED {
        if let Some(value) = sensor.remove(old) {
            subsensor.insert(new, value);
            moved.push((old, new));
        }
    }
    if subsensor.is_empty() {
        return;
    }
    let list_at = at.child("subsensors");
    if let Some(list) = sensor.get_mut("subsensors").and_then(|s| s.as_array_mut()) {
        // The file's own subsensors move one place on: the last first, so that the move
        // undone for a finding is the one of its own element.
        for index in (0..list.len()).rev() {
            told.moved(list_at.index(index), list_at.index(index + 1));
        }
        list.insert(0, subsensor);
    } else {
        // A `subsensors` that is no list has no subsensor to keep: the game's step writes
        // the list of the one it makes.
        if sensor.get("subsensors").is_some_and(|had| !had.is_null()) {
            told.wrong_type(list_at.clone());
        }
        let mut list = VArray::new();
        list.push(subsensor);
        sensor.insert("subsensors", list);
    }
    let first = list_at.index(0);
    for (old, new) in moved {
        told.moved(at.child(old), first.child(new));
    }
}
