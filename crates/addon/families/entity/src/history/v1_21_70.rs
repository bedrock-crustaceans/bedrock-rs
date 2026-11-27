//! Format 1.21.70: the struct and the step to 1.21.80.

use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};
use facet_value::{VArray, VObject};

use super::Payload;
use super::ops::{Members, Told, section_b, sections};

/// An entity document as written in format 1.21.70 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 21, 70>;

/// 1.21.70 → 1.21.80 (gate 1.21.80).
impl LegacyStep<1, 21, 70> for Payload {
    type Next = super::v1_21_80::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        if section_b(&payload, cx) {
            let mut misread = Vec::new();
            for section in sections(&mut payload) {
                if let Some(leashable) = section.members.object("minecraft:leashable")
                    && one_preset(leashable)
                {
                    // The crate's own note: a `presets` that is no array is replaced by the list
                    // of the one new preset; told at the member.
                    misread.push(section.at.with(&["minecraft:leashable", "presets"]));
                }
            }
            let mut told = Told::default();
            for at in misread {
                told.wrong_type(at);
            }
            told.report(cx);
        }
        Legacy(payload)
    }
}

/// B, `leashable`: `on_unleash_interact_only: true` when absent; the root distances move into
/// a new preset with `spring_type: "bouncy"`, appended to `presets`. Returns whether a
/// `presets` that is no array (and not `null`) was replaced.
fn one_preset(leashable: &mut VObject) -> bool {
    leashable.insert_absent("on_unleash_interact_only", true.into());
    let mut preset = VObject::new();
    for key in ["soft_distance", "hard_distance", "max_distance"] {
        if let Some(value) = leashable.remove(key) {
            preset.insert(key, value);
        }
    }
    preset.insert("spring_type", "bouncy");
    let mut replaced = false;
    if !leashable.get("presets").is_some_and(Value::is_array) {
        replaced = leashable
            .remove("presets")
            .is_some_and(|old| !old.is_null());
        leashable.insert("presets", VArray::new());
    }
    if let Some(presets) = leashable.get_mut("presets").and_then(|p| p.as_array_mut()) {
        presets.push(preset);
    }
    replaced
}
