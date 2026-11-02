//! Vanilla content as the oracle: every component of this module in the entity files of
//! bedrock-samples, the GDK retail packs and the BDS install reads without an error-level
//! finding, strictly at or above its typed version and through the component-side operations
//! and the lenient conversion below it. Skipped with a message when `BEDROCK_ADDON_CORPUS` is
//! unset.

use std::path::{Path, PathBuf};

use bedrock_addon_core::diagnostic::{Diagnostics, Severity};
use bedrock_addon_core::document::Component;
use bedrock_addon_core::json::{JsonSlice, Value, to_json};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_family_entity::component::older::{
    ApplyKnockbackRulesV1_8_0, PushableByEntityV1_26_20, PushableByEntityV1_26_30,
};
use bedrock_addon_family_entity::component::upgrade;
use bedrock_addon_family_entity::component::{EntityComponent, ExperienceReward, PushableByEntity};

use super::read;

/// A format version as three numbers.
type Version = (u16, u16, u16);

/// The struct version from which each key of this module is typed.
const FIRST_TYPED: &[(&str, Version)] = &[
    ("addrider", (1, 21, 130)),
    ("admire_item", (1, 26, 40)),
    ("anger_level", (1, 26, 50)),
    ("annotation.break_door", (1, 26, 60)),
    ("annotation.open_door", (1, 26, 40)),
    ("apply_knockback_rules", (1, 8, 0)),
    ("attack", (1, 26, 60)),
    ("barter", (1, 26, 50)),
    ("block_climber", (1, 26, 20)),
    ("block_movement_slowdown_immunity", (1, 26, 50)),
    ("body_rotation_always_follows_head", (1, 21, 120)),
    ("body_rotation_blocked", (1, 21, 120)),
    ("boss", (1, 26, 50)),
    ("bucketable", (1, 26, 40)),
    ("buoyant", (1, 26, 60)),
    ("burns_in_daylight", (1, 21, 130)),
    ("can_climb", (1, 21, 120)),
    ("can_fly", (1, 21, 120)),
    ("can_join_raid", (1, 26, 20)),
    ("can_power_jump", (1, 21, 120)),
    ("can_stand_on_powder_snow", (1, 26, 50)),
    ("cannot_be_attacked", (1, 26, 40)),
    ("collision_box", (1, 26, 50)),
    ("damage_over_time", (1, 26, 50)),
    ("dash_action", (1, 26, 60)),
    ("dimension_bound", (1, 26, 20)),
    ("economy_trade_table", (1, 26, 60)),
    ("exhaustion_values", (1, 26, 50)),
    ("experience_reward", (1, 21, 110)),
    ("fire_immune", (1, 21, 120)),
    ("floats_in_liquid", (1, 26, 40)),
    ("flocking", (1, 26, 50)),
    ("freezing_immune", (1, 26, 50)),
    ("freezing_vulnerable", (1, 26, 50)),
    ("game_event_movement_tracking", (1, 26, 50)),
    ("grows_crop", (1, 26, 50)),
    ("healable", (1, 26, 60)),
    ("hide", (1, 26, 40)),
    ("home", (1, 26, 50)),
    ("ignore_cannot_be_attacked", (1, 26, 40)),
    ("input_ground_controlled", (1, 21, 120)),
    ("insomnia", (1, 26, 50)),
    ("instant_despawn", (1, 26, 50)),
    ("interact", (1, 26, 50)),
    ("inventory", (1, 26, 60)),
    ("is_baby", (1, 21, 120)),
    ("is_charged", (1, 21, 120)),
    ("is_chested", (1, 21, 120)),
    ("is_hidden_when_invisible", (1, 21, 120)),
    ("is_ignited", (1, 21, 120)),
    ("is_illager_captain", (1, 21, 120)),
    ("is_pregnant", (1, 21, 120)),
    ("is_saddled", (1, 21, 120)),
    ("is_shaking", (1, 21, 120)),
    ("is_sheared", (1, 21, 120)),
    ("is_stackable", (1, 21, 120)),
    ("is_stunned", (1, 21, 120)),
    ("is_tamed", (1, 21, 120)),
    ("item_hopper", (1, 26, 20)),
    ("jump.dynamic", (1, 26, 40)),
    ("jump.static", (1, 26, 40)),
    ("managed_wandering_trader", (1, 26, 40)),
    ("movement.amphibious", (1, 26, 60)),
    ("movement.basic", (1, 26, 60)),
    ("movement.fly", (1, 26, 60)),
    ("movement.generic", (1, 26, 60)),
    ("movement.glide", (1, 26, 60)),
    ("movement.hover", (1, 26, 60)),
    ("movement.jump", (1, 26, 60)),
    ("movement.skip", (1, 26, 60)),
    ("movement.sway", (1, 26, 60)),
    ("not_pickable_from_inside", (1, 26, 40)),
    ("on_death", (1, 26, 40)),
    ("on_equipment_changed", (1, 26, 20)),
    ("on_friendly_anger", (1, 26, 40)),
    ("on_hurt", (1, 26, 40)),
    ("on_hurt_by_player", (1, 26, 40)),
    ("on_ignite", (1, 26, 40)),
    ("on_start_landing", (1, 26, 40)),
    ("on_start_takeoff", (1, 26, 40)),
    ("on_target_acquired", (1, 26, 40)),
    ("on_target_escape", (1, 26, 40)),
    ("on_wake_with_owner", (1, 26, 40)),
    ("out_of_control", (1, 26, 20)),
    ("persistent", (1, 26, 40)),
    ("physics", (1, 26, 50)),
    ("preferred_path", (1, 26, 50)),
    ("projectile", (1, 26, 50)),
    ("pushable_by_block", (1, 26, 60)),
    ("pushable_by_entity", (1, 26, 20)),
    ("rail_movement", (1, 26, 60)),
    ("renders_when_invisible", (1, 21, 120)),
    ("rideable", (1, 26, 60)),
    ("rotation_axis_aligned", (1, 21, 120)),
    ("rotation_locked_to_vehicle", (1, 21, 120)),
    ("scale_by_age", (1, 26, 60)),
    ("shareables", (1, 26, 60)),
    ("spawn_on_death", (1, 26, 20)),
    ("spell_effects", (1, 26, 60)),
    ("strength", (1, 26, 60)),
    ("suspect_tracking", (1, 26, 20)),
    ("tamemount", (1, 26, 50)),
    ("teleport", (1, 26, 60)),
    ("tick_world", (1, 21, 100)),
    ("timer", (1, 26, 60)),
    ("trade_resupply", (1, 26, 40)),
    ("trade_table", (1, 26, 50)),
    ("trail", (1, 26, 60)),
    ("transformation", (1, 26, 60)),
    ("transient", (1, 26, 20)),
    ("trust", (1, 26, 40)),
    ("uses_legacy_ambient_sounds", (1, 26, 50)),
    ("uses_legacy_friction", (1, 26, 20)),
    ("uses_uniform_air_drag", (1, 26, 20)),
    ("vibration_damper", (1, 26, 20)),
    ("vibration_listener", (1, 26, 20)),
    ("wants_jockey", (1, 26, 40)),
    ("wither_target_highest_damage", (1, 26, 30)),
];

/// The corpus root, or `None` (with a message) when the variable is unset.
fn corpus() -> Option<PathBuf> {
    let root = std::env::var_os("BEDROCK_ADDON_CORPUS").map(PathBuf::from);
    if root.is_none() {
        eprintln!("BEDROCK_ADDON_CORPUS is unset: the vanilla component sweep is skipped");
    }
    root
}

/// Every `entities/*.json` below `dir`.
fn entity_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            entity_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "json")
            && path
                .parent()
                .and_then(Path::file_name)
                .is_some_and(|p| p == "entities")
        {
            out.push(path);
        }
    }
}

/// `"1.21.0"` as numbers.
fn version(text: &str) -> Option<Version> {
    let mut parts = text.split('.').map(|p| p.parse::<u16>().ok());
    Some((
        parts.next()??,
        parts.next()??,
        parts.next().flatten().unwrap_or(0),
    ))
}

/// The JSON text of `value`.
fn text(value: &Value) -> String {
    to_json(value).expect("writes")
}

/// Runs the component-side operations whose gate is above `at` on `value` (`name` without
/// `minecraft:`), oldest first, as the steps do.
fn upgrade_json(name: &str, at: Version, value: &mut Value, d: &mut Diagnostics) {
    match name {
        "interact" => {
            if at < (1, 13, 0) {
                upgrade::interact_1_13_0(value, d);
            }
            if at < (1, 20, 80) {
                upgrade::interact_slots_1_20_80(value);
            }
            if at < (1, 26, 0) {
                upgrade::interact_swing_1_26_0(value);
            }
        }
        "home" if at < (1, 21, 40) => upgrade::home_1_21_40(value),
        "projectile" => {
            if at < (1, 21, 120) {
                upgrade::projectile_1_21_120(value);
            }
            if at < (1, 26, 40) {
                upgrade::projectile_1_26_40(value, d);
            }
        }
        "jump.dynamic" if at < (1, 26, 0) => upgrade::jump_dynamic_1_26_0(value),
        _ => {}
    }
}

/// Reads one component as the chain would for a document at `at`; returns whether it was kept.
fn read_component(name: &str, at: Version, value: &Value, d: &mut Diagnostics) -> bool {
    let typed = FIRST_TYPED
        .iter()
        .find(|(key, _)| *key == name)
        .map_or((9999, 0, 0), |(_, v)| *v);
    let mode = if at >= typed {
        ConvertMode::Strict
    } else {
        ConvertMode::Lenient
    };
    let mut value = value.clone();
    upgrade_json(name, at, &mut value, d);
    let json = text(&value);
    let slice = JsonSlice {
        text: &json,
        offset: 0,
    };
    match name {
        "apply_knockback_rules" if at < (1, 26, 30) => {
            ApplyKnockbackRulesV1_8_0::read(slice, mode, &LoadContext::detached(), d).is_some()
        }
        "pushable_by_entity" if at < (1, 26, 30) => {
            PushableByEntityV1_26_20::read(slice, mode, &LoadContext::detached(), d).is_some()
        }
        "pushable_by_entity" if at < (1, 26, 40) => {
            PushableByEntityV1_26_30::read(slice, mode, &LoadContext::detached(), d).is_some()
        }
        "experience_reward" if at < (1, 26, 0) => {
            ExperienceReward::read_v1_21_110(slice, mode, &LoadContext::detached(), d).is_some()
        }
        _ => {
            let (kept, found) = read(&format!("minecraft:{name}"), &json, mode);
            d.extend(found);
            kept.is_some()
        }
    }
}

/// The component sections of an entity file: `components` and each component group.
fn sections(entity: &Value) -> Vec<(String, &Value)> {
    let mut out = Vec::new();
    let Some(entity) = entity.as_object() else {
        return out;
    };
    if let Some(components) = entity.get("components") {
        out.push(("components".to_owned(), components));
    }
    if let Some(groups) = entity.get("component_groups").and_then(Value::as_object) {
        for (name, group) in groups.iter() {
            out.push((format!("component_groups/{}", name.as_str()), group));
        }
    }
    out
}

#[test]
fn vanilla_entity_files_read_this_modules_components_without_an_error() {
    let Some(root) = corpus() else { return };
    let mut files = Vec::new();
    for source in ["bedrock-samples", "gdk-retail", "bds-1.26.36.1"] {
        entity_files(&root.join(source), &mut files);
    }
    let mut read_count = 0usize;
    let mut skipped = 0usize;
    let mut failures = Vec::new();
    let mut warnings = std::collections::BTreeMap::new();
    for file in &files {
        let Ok(bytes) = std::fs::read(file) else {
            continue;
        };
        let Ok(text) = std::str::from_utf8(&bytes) else {
            skipped += 1;
            continue;
        };
        let text = text.trim_start_matches('\u{feff}');
        let Ok(document) = facet_json::from_str_jsonc::<Value>(text) else {
            skipped += 1;
            continue;
        };
        let Some(document) = document.as_object() else {
            continue;
        };
        let Some(at) = document
            .get("format_version")
            .and_then(Value::as_string)
            .and_then(|v| version(v.as_str()))
        else {
            continue;
        };
        let Some(entity) = document.get("minecraft:entity") else {
            continue;
        };
        for (section, components) in sections(entity) {
            let Some(components) = components.as_object() else {
                continue;
            };
            for (key, value) in components.iter() {
                let Some(name) = key.as_str().strip_prefix("minecraft:") else {
                    continue;
                };
                if !FIRST_TYPED.iter().any(|(k, _)| *k == name) {
                    continue;
                }
                let mut d = Diagnostics::default();
                let kept = read_component(name, at, value, &mut d);
                read_count += 1;
                if !kept || d.iter().any(|f| f.severity == Severity::Error) {
                    failures.push(format!(
                        "{} {section}/{}: kept {kept}, {}",
                        file.display(),
                        key.as_str(),
                        d.summary()
                    ));
                }
                for finding in d.iter().filter(|f| f.severity == Severity::Warning) {
                    *warnings
                        .entry(format!(
                            "{name}/{} {}",
                            finding.location.json_path, finding.code
                        ))
                        .or_insert(0usize) += 1;
                }
            }
        }
    }
    eprintln!(
        "{} files, {read_count} components read, {skipped} files not JSON text",
        files.len()
    );
    for (warning, count) in &warnings {
        eprintln!("{count:6} {warning}");
    }
    assert!(read_count > 1000, "the sweep read {read_count} components");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn v11_retail_sulfur_cube_presets_upgrade_to_the_retail_1_26_40_copy() {
    // v11: "`sulfur_cube` 1.26.30 → 1.26.40 carries `require_collision_overlap:
    // false` equal to the retail 1.26.40 copy".
    let Some(root) = corpus() else { return };
    let packs = root.join("gdk-retail/behavior_packs");
    let load = |pack: &str| {
        let path = packs.join(pack).join("entities/sulfur_cube.json");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        facet_json::from_str_jsonc::<Value>(&text).expect("JSON")
    };
    let (old, new) = (load("vanilla_1.26.30"), load("vanilla_1.26.40"));
    let pushables = |document: &Value| -> Vec<(String, Value)> {
        let entity = document
            .as_object()
            .and_then(|o| o.get("minecraft:entity"))
            .expect("an entity");
        sections(entity)
            .into_iter()
            .filter_map(|(section, components)| {
                let value = components.as_object()?.get(PushableByEntity::NAME)?;
                Some((section, value.clone()))
            })
            .collect()
    };
    let (old, new) = (pushables(&old), pushables(&new));
    let mut compared = 0;
    for (section, value) in &old {
        let written = text(value);
        let mut d = Diagnostics::default();
        let slice = JsonSlice {
            text: &written,
            offset: 0,
        };
        let before = PushableByEntityV1_26_30::read(
            slice,
            ConvertMode::Strict,
            &LoadContext::detached(),
            &mut d,
        )
        .expect("the 1.26.30 component reads");
        if before.presets.is_empty() {
            continue;
        }
        let upgraded = PushableByEntity::from(before);
        let (_, retail) = new
            .iter()
            .find(|(s, _)| s == section)
            .unwrap_or_else(|| panic!("{section} in 1.26.40"));
        let (retail, d) = read(PushableByEntity::NAME, &text(retail), ConvertMode::Strict);
        assert!(!d.has_errors(), "{d:?}");
        let Some(EntityComponent::PushableByEntity(retail)) = retail else {
            panic!("typed")
        };
        assert_eq!(upgraded.presets, retail.presets, "{section}");
        compared += 1;
    }
    assert!(compared >= 3, "compared {compared} preset lists");
}
