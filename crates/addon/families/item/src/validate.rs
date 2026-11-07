//! The item validators on the newest struct: the rules vanilla checks after the components
//! are read (identifier, use duration, durability dependants, storage, kinetic weapons, swing
//! sounds, the wearable stack size) and the crate's own check of trigger events.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::version::{FormatVersion, known};

use super::component::{
    BundleInteraction, DamageAbsorption, Durability, DurabilitySensor, Food, KineticWeapon,
    MaxStackSize, Shooter, StorageItem, SwingSounds, UseModifiers, Wearable, WearableSlot,
};
use super::legacy::ItemFormat;
use super::{Item, codes};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, Severity,
};
use bedrock_addon_core::id::syntax::has_namespace;
use bedrock_addon_core::load::LoadContext;

/// A validator finding at `at/<path…>`.
fn push(
    d: &mut Diagnostics,
    at: &JsonPath,
    path: &[&str],
    code: DiagnosticCode,
    severity: Severity,
) {
    let mut full = at.clone();
    for part in path {
        full.push(part);
    }
    d.push(
        Diagnostic::error(code)
            .severity(severity)
            .at(full)
            .phase(Phase::Validate),
    );
}

/// The path of the component `name`.
fn components(name: &str) -> [&str; 2] {
    ["components", name]
}

/// Runs the validators on `item`; findings go below `at` (the payload's path). An error
/// rejects the item.
#[allow(
    clippy::too_many_lines,
    reason = "one rule after the other, in the order of the rules"
)]
pub(crate) fn validate(
    item: &Item,
    original: FormatVersion,
    cx: &LoadContext,
    at: &JsonPath,
    d: &mut Diagnostics,
) {
    let _ = cx;
    // The identifier is `ns:name` (a `minecraft:` item of another pack is judged on the stack).
    if !has_namespace(item.description.identifier.as_str()) {
        push(
            d,
            at,
            &["description", "identifier"],
            DiagnosticCode::from(&codes::IDENTIFIER),
            Severity::Error,
        );
    }
    triggers(item, at, d);
    if item.legacy.format == ItemFormat::Legacy {
        return;
    }
    let c = &item.components;
    // Food and shooters need a use duration.
    let legacy_duration = item.legacy.use_duration_ticks().is_some_and(|t| t != 0);
    let duration = c
        .get::<UseModifiers>()
        .is_some_and(|m| m.use_duration > 0.0)
        || legacy_duration;
    for (present, name) in [
        (c.contains::<Food>(), "minecraft:food"),
        (c.contains::<Shooter>(), "minecraft:shooter"),
    ] {
        if present && !duration {
            push(
                d,
                at,
                &components(name),
                codes::USE_DURATION_REQUIRED.with([name]).into(),
                // The game's validator logs this with validation code 1, which the content
                // log writes at the warning level and which does not make the load fail: a
                // warning, the item loads.
                Severity::Warning,
            );
        }
    }
    // Components that depend on a durability.
    let durable = c.get::<Durability>().is_some_and(|d| d.max_durability > 0);
    for (present, name) in [
        (
            c.contains::<DamageAbsorption>(),
            "minecraft:damage_absorption",
        ),
        (
            c.contains::<DurabilitySensor>(),
            "minecraft:durability_sensor",
        ),
    ] {
        if present && !durable {
            push(
                d,
                at,
                &components(name),
                codes::DURABILITY_REQUIRED.with([name]).into(),
                // Validation code 1 again: a warning, the item loads.
                Severity::Warning,
            );
        }
    }
    // Storage components.
    if c.contains::<BundleInteraction>() && !c.contains::<StorageItem>() {
        push(
            d,
            at,
            &components("minecraft:bundle_interaction"),
            DiagnosticCode::from(&codes::BUNDLE_NEEDS_STORAGE),
            // Validation code 1 again: a warning, the item loads.
            Severity::Warning,
        );
    }
    if c.contains::<StorageItem>() && c.get::<MaxStackSize>().is_none_or(|m| m.value != 1) {
        // The storage validator uses validation code 2, which the content log writes at the
        // error level and which makes the load fail: the item is refused.
        push(
            d,
            at,
            &components("minecraft:storage_item"),
            DiagnosticCode::from(&codes::STORAGE_NEEDS_STACK_OF_ONE),
            Severity::Error,
        );
    }
    // Kinetic weapons.
    if let Some(kinetic) = c.get::<KineticWeapon>() {
        kinetic_rules(kinetic, c.contains::<UseModifiers>(), at, d);
    }
    // Swing sounds need at least one sound.
    if let Some(sounds) = c.get::<SwingSounds>()
        && sounds.attack_miss.is_none()
        && sounds.attack_hit.is_none()
        && sounds.attack_critical_hit.is_none()
    {
        push(
            d,
            at,
            &components("minecraft:swing_sounds"),
            DiagnosticCode::from(&codes::SWING_SOUNDS_EMPTY),
            Severity::Error,
        );
    }
    // A wearable defaults to a stack of one (as of 1.26.45; which slots count is inferred, and
    // so is that only a 1.26.30 or later document gets it: a 1.20.10 item has no such default).
    if original >= known::V1_26_30
        && let Some(wearable) = c.get::<Wearable>()
        && !c.contains::<MaxStackSize>()
        && wearable.slot != WearableSlot::Offhand
    {
        // The game logs this at the content log's error level and the item loads: a kept
        // error.
        let full = at.child("components").child("minecraft:wearable");
        d.push_kept_error(
            Diagnostic::error(
                codes::WEARABLE_STACK_SIZE
                    .with(["minecraft:wearable", wearable.slot.name()])
                    .into(),
            )
            .at(full)
            .phase(Phase::Validate),
        );
    }
}

/// The four checks of a kinetic weapon: it needs `use_modifiers`, at least one condition, no
/// zero `max_duration`, and not both speed members in one condition.
fn kinetic_rules(kinetic: &KineticWeapon, use_modifiers: bool, at: &JsonPath, d: &mut Diagnostics) {
    let path = ["components", "minecraft:kinetic_weapon"];
    if !use_modifiers {
        push(
            d,
            at,
            &path,
            DiagnosticCode::from(&codes::KINETIC_NEEDS_USE_MODIFIERS),
            Severity::Error,
        );
    }
    let conditions = [
        ("damage_conditions", &kinetic.damage_conditions),
        ("knockback_conditions", &kinetic.knockback_conditions),
        ("dismount_conditions", &kinetic.dismount_conditions),
    ];
    if conditions.iter().all(|(_, c)| c.is_none()) {
        push(
            d,
            at,
            &path,
            DiagnosticCode::from(&codes::KINETIC_NEEDS_CONDITION),
            Severity::Error,
        );
    }
    for (name, condition) in conditions {
        let Some(condition) = condition else {
            continue;
        };
        if condition.max_duration == Some(0) {
            push(
                d,
                at,
                &[path[0], path[1], name, "max_duration"],
                codes::KINETIC_ZERO_DURATION.with([name, name]).into(),
                Severity::Error,
            );
        }
        if condition.min_speed.is_some() && condition.min_relative_speed.is_some() {
            push(
                d,
                at,
                &[path[0], path[1], name],
                codes::KINETIC_SPEED_EXCLUSIVE.with([name]).into(),
                Severity::Error,
            );
        }
    }
}

/// A trigger of the item that fires on the item's holder itself names an event of the
/// item's own `events`; one that names none is an information, the trigger kept. Read are the
/// triggers of the side-car (`__deprecated_components`) and of `minecraft:digger`
/// (`__deprecated_on_dig`).
fn triggers(item: &Item, at: &JsonPath, d: &mut Diagnostics) {
    use bedrock_addon_core::shared::{EventTrigger, FilterSubject};

    use super::component::Digger;

    let events = item
        .events
        .as_ref()
        .and_then(bedrock_addon_core::json::Value::as_object);
    let mut check = |path: &[&str], index: Option<usize>, rest: &[&str], trigger: &EventTrigger| {
        let Some(event) = trigger.event.as_deref().filter(|event| !event.is_empty()) else {
            return;
        };
        let own = matches!(trigger.target, None | Some(FilterSubject::Self_));
        if !own || events.is_some_and(|events| events.contains_key(event)) {
            return;
        }
        let mut full = at.clone();
        for part in path {
            full.push(part);
        }
        if let Some(index) = index {
            full.push_index(index);
        }
        for part in rest {
            full.push(part);
        }
        if !trigger.bare {
            full.push("event");
        }
        // The game has no line for this, so the information is the crate's own.
        d.push(
            Diagnostic::of(&core_codes::UNRESOLVED_REFERENCE)
                .at(full)
                .phase(Phase::Validate)
                .detail(
                    bedrock_addon_core::diagnostic::DiagnosticDetail::Reference {
                        kind: "event",
                        id: event.to_owned(),
                        hint: None,
                    },
                ),
        );
    };
    let deprecated = &item.deprecated;
    for (outer, inner, trigger) in [
        (
            "on_consume_trigger",
            "on_consume_trigger",
            &deprecated.on_consume,
        ),
        ("on_use", "on_use_trigger", &deprecated.on_use),
        ("on_use_on_trigger", "on_use_on", &deprecated.on_use_on),
        (
            "on_hurt_actor_trigger",
            "on_hurt_actor",
            &deprecated.on_hurt_actor,
        ),
        (
            "on_hit_actor_trigger",
            "on_hit_actor",
            &deprecated.on_hit_actor,
        ),
        (
            "on_hit_block_trigger",
            "on_hit_block",
            &deprecated.on_hit_block,
        ),
        (
            "on_complete_trigger",
            "on_complete",
            &deprecated.on_complete,
        ),
    ] {
        if let Some(trigger) = trigger {
            check(
                &["__deprecated_components", outer, inner],
                None,
                &[],
                trigger,
            );
        }
    }
    if let Some(digger) = item.components.get::<Digger>() {
        let base = ["components", "minecraft:digger"];
        if let Some(trigger) = &digger.deprecated_on_dig {
            check(&base, None, &["__deprecated_on_dig"], trigger);
        }
        for (index, speed) in digger.destroy_speeds.iter().enumerate() {
            if let Some(trigger) = &speed.deprecated_on_dig {
                let path = [base[0], base[1], "destroy_speeds"];
                check(&path, Some(index), &["__deprecated_on_dig"], trigger);
            }
        }
    }
}
