//! Format 1.18.0: `block_collision` and `aim_collision` (the 1.18.0 renames),
//! `block_light_filter`, `crafting_table` without `grid_size`.

use indexmap::IndexMap;

use super::legacy::{BlockPayload, component_path, input_check_failed, number};
use super::ops::{clamp, int, missing_field, number_member, real, unwrap, whole};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Phase, VanillaText};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx, Versioned};

/// A block document of `[1.18.0, 1.19.10)`.
pub type Block = Legacy<BlockPayload, 1, 18, 0>;

/// The 1.19.10 unwraps: component, the member holding its value.
const UNWRAPS: [(&str, &str); 9] = [
    ("minecraft:destroy_time", "destroy_time"),
    ("minecraft:explosion_resistance", "resistance"),
    ("minecraft:display_name", "display_name"),
    ("minecraft:friction", "friction"),
    ("minecraft:geometry", "geometry"),
    ("minecraft:loot", "loot_table"),
    ("minecraft:map_color", "map_color"),
    ("minecraft:block_light_filter", "filter_level"),
    ("minecraft:breathability", "breathing_type"),
];

/// Vanilla's 1.19.10 fixers.
impl LegacyStep<1, 18, 0> for BlockPayload {
    type Next = super::v1_19_10::Block;

    #[allow(
        clippy::too_many_lines,
        reason = "one op after another, in the order of the fixers"
    )]
    fn step(this: Block, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut block = this.0;
        let to = <Self::Next as Versioned>::VERSION;
        // Input checks on the base.
        if let Some(emission) = block.components.get("minecraft:block_light_emission")
            && !emission_value(emission).is_some_and(|n| (0.0..=1.0).contains(&n))
        {
            input_check_failed(
                "minecraft:block_light_emission",
                to,
                cx.original_version,
                cx.diagnostics,
            );
        }
        if let Some(filter) = block.components.get("minecraft:block_light_filter")
            && !number_member(filter, "filter_level").is_some_and(|n| (0.0..=15.0).contains(&n))
        {
            input_check_failed(
                "minecraft:block_light_filter",
                to,
                cx.original_version,
                cx.diagnostics,
            );
        }
        // `block_collision` → `collision_box`, `aim_collision` in place: `enabled: true` is
        // removed, `enabled: false` makes the value `false`.
        let d = &mut *cx.diagnostics;
        let mut collision = Vec::new();
        for (section, map) in block.sections_mut() {
            for name in ["minecraft:block_collision", "minecraft:aim_collision"] {
                if let Some(value) = map.get_mut(name) {
                    collapse_enabled(value, section, name, &mut collision);
                }
            }
        }
        // The game's collision upgrades drop the members with no log (the text below is the
        // unwrap's): the crate reports it as an information.
        for (section, name) in collision {
            d.push(
                Diagnostic::info(DiagnosticCode::DroppedField { text: None })
                    .at(component_path(section, name))
                    .phase(Phase::Upgrade),
            );
        }
        block.rename_component(
            "minecraft:block_collision",
            "minecraft:collision_box",
            |v| v,
            d,
        );
        // The unwraps.
        for (section, map) in block.sections_mut() {
            for (name, field) in UNWRAPS {
                let Some(value) = map.get(name) else { continue };
                if let Some(inner) = unwrap(value, field) {
                    // The object form is read for `field` alone; the others go with the
                    // unwrap.
                    if value.as_object().is_some_and(|object| object.len() > 1) {
                        thrown_away(section, name, d);
                    }
                    if value.is_object() {
                        // The value was the member `field` of the file's object form.
                        let mut from = component_path(section, name);
                        from.push(field);
                        d.moved(from, component_path(section, name));
                    }
                    map.insert(name.to_owned(), inner);
                } else {
                    map.shift_remove(name);
                    missing_field(section, name, field, d);
                }
            }
            convert_crafting_table(map, section, d);
            convert_flammable(map, section, d);
        }
        // `block_light_emission` → `light_emission`: `trunc(f * 15)`, clamped to 0..=15 (the
        // clamp is the crate's own; the game leaves the value unclamped).
        if block.has_anywhere("minecraft:block_light_emission") {
            // The game's missing-field check on `emission` comes first: an object form
            // without it goes, with the game's line.
            let mut missing = Vec::new();
            for (section, map) in block.sections_mut() {
                if map
                    .get("minecraft:block_light_emission")
                    .and_then(Value::as_object)
                    .is_some_and(|object| !object.contains_key("emission"))
                {
                    map.shift_remove("minecraft:block_light_emission");
                    missing.push(section);
                }
            }
            for section in missing {
                missing_field(section, "minecraft:block_light_emission", "emission", d);
            }
            // The object form is read for `emission` alone; another member
            // of it goes with the conversion, told at the member.
            for (section, map) in block.sections_mut() {
                let Some(object) = map
                    .get("minecraft:block_light_emission")
                    .and_then(Value::as_object)
                else {
                    continue;
                };
                if object.contains_key("emission") && object.len() > 1 {
                    thrown_away(section, "minecraft:block_light_emission", d);
                }
            }
            block.rename_component(
                "minecraft:block_light_emission",
                "minecraft:light_emission",
                |v| match emission_value(&v) {
                    Some(f) => real((f * 15.0).trunc()),
                    None => v,
                },
                d,
            );
            for (section, map) in block.sections_mut() {
                if let Some(value) = map.get_mut("minecraft:light_emission")
                    && let Some(n) = number(value)
                {
                    *value = int(whole(clamp(
                        n,
                        0.0,
                        15.0,
                        section,
                        "minecraft:light_emission",
                        d,
                    )));
                }
            }
        }
        // `ticking` → `queued_ticking`.
        let mut dropped = Vec::new();
        for (section, map) in block.sections_mut() {
            if let Some(value) = map.get_mut("minecraft:ticking")
                && let Err(field) = convert_ticking(value, section, d)
            {
                dropped.push((section, field));
                map.shift_remove("minecraft:ticking");
            }
        }
        for (section, field) in dropped {
            missing_field(section, "minecraft:ticking", field, d);
        }
        block.rename_component("minecraft:ticking", "minecraft:queued_ticking", |v| v, d);
        Legacy(block)
    }
}

/// "Members of component %s will be thrown away as part of the upgrade": the game logs it at
/// the content log's error level when the object has its value member and another, and goes
/// on with the value: a kept error at the component.
fn thrown_away(section: Option<usize>, name: &str, d: &mut Diagnostics) {
    d.push_kept_error(
        Diagnostic::error(DiagnosticCode::DroppedField {
            text: Some(VanillaText::Formatted {
                template: "Members of component {} will be thrown away as part of the upgrade",
                args: vec![name.to_owned()],
            }),
        })
        .at(component_path(section, name))
        .phase(Phase::Upgrade),
    );
}

/// The `enabled` handling of the collision components.
fn collapse_enabled(
    value: &mut Value,
    section: Option<usize>,
    name: &'static str,
    log: &mut Vec<(Option<usize>, &'static str)>,
) {
    let Some(object) = value.as_object() else {
        return;
    };
    match object.get("enabled").and_then(Value::as_bool) {
        Some(false) => {
            *value = Value::FALSE;
            log.push((section, name));
        }
        Some(true) => super::ops::edit_object(value, |o| {
            o.shift_remove("enabled");
        }),
        None => {}
    }
}

/// `crafting_table`: dropped without `crafting_tags`; `custom_description` → `table_name`.
fn convert_crafting_table(
    map: &mut IndexMap<String, Value>,
    section: Option<usize>,
    d: &mut Diagnostics,
) {
    const NAME: &str = "minecraft:crafting_table";
    let Some(value) = map.get_mut(NAME) else {
        return;
    };
    let Some(mut table) = super::legacy::object(value) else {
        return;
    };
    if !table.contains_key("crafting_tags") {
        map.shift_remove(NAME);
        missing_field(section, NAME, "crafting_tags", d);
        return;
    }
    super::legacy::rename_member(
        &mut table,
        section,
        NAME,
        "custom_description",
        "table_name",
        d,
    );
    *value = super::to_object(&table);
}

/// `flammable`: both odds required; `flame_odds` → `catch_chance_modifier`, `burn_odds` →
/// `destroy_chance_modifier`.
fn convert_flammable(
    map: &mut IndexMap<String, Value>,
    section: Option<usize>,
    d: &mut Diagnostics,
) {
    const NAME: &str = "minecraft:flammable";
    let Some(value) = map.get_mut(NAME) else {
        return;
    };
    let Some(mut flammable) = super::legacy::object(value) else {
        return;
    };
    for field in ["flame_odds", "burn_odds"] {
        // A member written `null` is there for the game's check; the typed read
        // that follows takes it as absent, the default: told where it is.
        match flammable.get(field) {
            None => {
                map.shift_remove(NAME);
                missing_field(section, NAME, field, d);
                return;
            }
            Some(odds) if odds.is_null() => {
                let mut at = component_path(section, NAME);
                at.push(field);
                super::legacy::misread(&at, d);
            }
            Some(_) => {}
        }
    }
    for (old, new) in [
        ("flame_odds", "catch_chance_modifier"),
        ("burn_odds", "destroy_chance_modifier"),
    ] {
        super::legacy::rename_member(&mut flammable, section, NAME, old, new, d);
    }
    *value = super::to_object(&flammable);
}

/// `block_light_emission` as a number: the bare value or `emission`, whose default is 0 in the
/// object form.
fn emission_value(value: &Value) -> Option<f64> {
    match value.as_object() {
        Some(object) => match object.get("emission") {
            Some(emission) => super::legacy::number(emission),
            None => Some(0.0),
        },
        None => super::legacy::number(value),
    }
}

/// What a number of seconds times 20 may be off a whole tick by and still be that tick
/// (`0.35 * 20` is not 7 to the last bit).
const TICK_TOLERANCE: f64 = 1e-6;

/// `ticking`: the missing member when `on_tick` or `range` is missing; otherwise `range` becomes
/// `interval_range`, each end clamped to `>= 0`, sorted, times 20 and rounded to a tick, and
/// bounded to the largest `u32` tick count (the crate's own bound).
/// An element of `range` that is no number is no end: it is left out, and told at the element
/// (the crate's own finding).
fn convert_ticking(
    value: &mut Value,
    section: Option<usize>,
    d: &mut Diagnostics,
) -> Result<(), &'static str> {
    // The game's missing-field check looks only at an object and only for the members'
    // presence; the conversion takes a `range` that is an array alone, and leaves another
    // as it is.
    let Some(mut ticking) = super::legacy::object(value) else {
        return Ok(());
    };
    if !ticking.contains_key("on_tick") {
        return Err("on_tick");
    }
    let Some(range) = ticking.get("range") else {
        return Err("range");
    };
    let Some(elements) = range.as_array() else {
        let mut at = component_path(section, "minecraft:ticking");
        at.push("range");
        super::legacy::misread(&at, d);
        return Ok(());
    };
    // Each end with the index the file has it at: bounded to `>= 0` and rounded to a whole
    // tick, both the crate's own and both told at the end; then sorted.
    let mut ends: Vec<(usize, f64)> = Vec::new();
    for (index, element) in elements.iter().enumerate() {
        let mut at = component_path(section, "minecraft:ticking");
        at.push("range");
        at.push_index(index);
        let Some(end) = number(element) else {
            super::legacy::misread(&at, d);
            continue;
        };
        // The product is stored as a `u32`: an end beyond it is bounded to
        // the largest tick count, as one below 0 is bounded to 0, and told the same way.
        let ticks = (end.max(0.0) * 20.0).min(f64::from(u32::MAX));
        if end < 0.0 || (end * 20.0).round() > f64::from(u32::MAX) {
            d.push(
                Diagnostic::info(DiagnosticCode::ClampedValue { text: None })
                    .at(at.clone())
                    .phase(Phase::Upgrade),
            );
        } else if (ticks - ticks.round()).abs() > TICK_TOLERANCE {
            d.push(
                Diagnostic::info(DiagnosticCode::InvalidValue { text: None })
                    .at(at.clone())
                    .phase(Phase::Upgrade),
            );
        }
        ends.push((index, ticks.round()));
    }
    ends.sort_by(|a, b| a.1.total_cmp(&b.1));
    let ticks: facet_value::VArray = ends.iter().map(|(_, n)| int(whole(*n))).collect();
    let index = ticking.get_index_of("range").unwrap_or(0);
    ticking.shift_remove("range");
    let mut interval = component_path(section, "minecraft:ticking");
    interval.push("interval_range");
    if ticking
        .get("interval_range")
        .is_some_and(|had| !had.is_null())
    {
        // The file's own `interval_range` beside the `range` it is made of.
        super::legacy::replaced(&interval, d);
    }
    ticking.shift_remove("interval_range");
    ticking.shift_insert(
        index.min(ticking.len()),
        "interval_range".to_owned(),
        ticks.into(),
    );
    let mut range = component_path(section, "minecraft:ticking");
    range.push("range");
    d.moved(range.clone(), interval.clone());
    // An end the sort (or an element left out) put at another index is named by the index
    // the file has it at.
    for (now, (was, _)) in ends.iter().enumerate() {
        let (mut from, mut to) = (range.clone(), interval.clone());
        from.push_index(*was);
        to.push_index(now);
        d.moved(from, to);
    }
    *value = super::to_object(&ticking);
    Ok(())
}
