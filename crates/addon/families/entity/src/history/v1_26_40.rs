//! Format 1.26.40: the struct and the step to 1.26.50, with the ungated
//! `using_door_annotation` op that runs once after the last step on the payload, and
//! the step's five upgrader ops.

use bedrock_addon_core::diagnostic::{DiagnosticCode, JsonPath, VanillaText};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};
use facet_value::VObject;

use bedrock_addon_core::vanilla::ActorType;

use super::Payload;
use super::fragments;
use super::ops::{Members, Told, components, finding, group, is, section_a, section_b, sections};

/// An entity document as written in format 1.26.40 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 26, 40>;

/// The version from which `using_door_annotation` is written `true`.
const DOOR_ANNOTATION_TRUE: FormatVersion = known::V1_10_0;

/// 1.26.40 → 1.26.50.
impl LegacyStep<1, 26, 40> for Payload {
    type Next = super::v1_26_50::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        let mut invalid: Vec<(String, JsonPath)> = Vec::new();
        for section in sections(&mut payload) {
            // `behavior.ranged_attack`'s `attack_range` / `in_range_movement_mode` conversion is
            // the goal's versioned parse, run by `into_latest`.
            if let Some(projectile) = section.members.object("minecraft:projectile")
                && let Some(written) = anchor(projectile)
            {
                invalid.push((
                    written,
                    section.at.with(&["minecraft:projectile", "anchor"]),
                ));
            }
        }
        for (value, at) in invalid {
            finding(
                cx,
                DiagnosticCode::InvalidValue {
                    text: Some(VanillaText::NotAValidValue {
                        value,
                        member: "anchor",
                    }),
                },
                &at,
            );
        }
        if section_a(&payload, cx) {
            if is(cx, &[ActorType::Enderman]) {
                enderman_teleport(&mut payload);
            }
            door_annotation(&mut payload, cx);
        }
        if section_b(&payload, cx) {
            components_1_26_50(&mut payload);
        }
        after_1_26_50(&mut payload, cx);
        super::v1_26_50::Entity(super::into_latest(&payload, cx))
    }
}

/// `projectile.anchor` as an integer: `0` → `"origin"`, `1` →
/// `"eye_height"`, `2` → `"middle"`; another integer falls back to `"origin"` and is returned
/// for its `InvalidValue` warning.
fn anchor(projectile: &mut VObject) -> Option<String> {
    let anchor = projectile.get_mut("anchor")?;
    let number = anchor.as_number().filter(|n| n.is_integer())?.to_i64()?;
    let (name, invalid) = match number {
        0 => ("origin", None),
        1 => ("eye_height", None),
        2 => ("middle", None),
        other => ("origin", Some(other.to_string())),
    };
    *anchor = name.into();
    invalid
}

/// The door-annotation support rule: each navigation component in `components` (groups are
/// not visited) gets `using_door_annotation` overwritten with whether the original version is
/// 1.10.0 or above.
///
/// The crate's own information, since the fixer does it: a value of the document that is replaced
/// by another is told.
fn door_annotation(payload: &mut Payload, cx: &mut UpgradeCx<'_>) {
    const NAVIGATION: [&str; 5] = [
        "minecraft:navigation.walk",
        "minecraft:navigation.generic",
        "minecraft:navigation.float",
        "minecraft:navigation.climb",
        "minecraft:navigation.swim",
    ];
    let Some(components) = &mut payload.components else {
        return;
    };
    let supported = cx.original_version >= DOOR_ANNOTATION_TRUE;
    let mut told = Told::default();
    for key in NAVIGATION {
        if let Some(navigation) = components.object(key) {
            if navigation
                .get("using_door_annotation")
                .is_some_and(|had| !had.is_null() && had.as_bool() != Some(supported))
            {
                told.dropped(JsonPath::new().with(&["components", key, "using_door_annotation"]));
            }
            navigation.insert("using_door_annotation", supported);
        }
    }
    told.report(cx);
}

/// `minecraft:uses_legacy_ambient_sounds`: set on every document this step upgrades.
const USES_LEGACY_AMBIENT_SOUNDS: &str = "minecraft:uses_legacy_ambient_sounds";
/// `minecraft:can_stand_on_powder_snow`: for the `lightweight` family.
const CAN_STAND_ON_POWDER_SNOW: &str = "minecraft:can_stand_on_powder_snow";
/// `minecraft:block_movement_slowdown_immunity`: cobwebs for the spiders.
const SLOWDOWN_IMMUNITY: &str = "minecraft:block_movement_slowdown_immunity";
/// `minecraft:freezing_immune`.
const FREEZING_IMMUNE: &str = "minecraft:freezing_immune";
/// `minecraft:freezing_vulnerable`.
const FREEZING_VULNERABLE: &str = "minecraft:freezing_vulnerable";
/// The mobs the step gives `freezing_immune`.
const FREEZING_IMMUNE_MOBS: [&str; 7] = [
    "minecraft:creaking",
    "minecraft:skeleton",
    "minecraft:wither",
    "minecraft:ender_dragon",
    "minecraft:polar_bear",
    "minecraft:snow_golem",
    "minecraft:stray",
];
/// The identifiers the step gives `freezing_vulnerable`.
const FREEZING_VULNERABLE_MOBS: [&str; 3] = [
    "minecraft:strider",
    "minecraft:blaze",
    "minecraft:magma_cube",
];
/// The sulfur cube's group that gets `freezing_immune`.
const SULFUR_CUBE_GROUP: &str = "minecraft:sulfur_cube_medium_with_block";

/// The five ops the game's 1.26.50 upgrader runs after the rewrites, on
/// every document this step upgrades: a cereal upgrader, so neither `do_not_upgrade` nor the
/// fixer's early exit stops it. A component the op inserts is the
/// default-constructed one, written as the 1.26.50 vanilla files write it.
///
/// The crate's own information, since the game does it without a word: a
/// `uses_legacy_ambient_sounds` the file wrote with something in it, which the step replaces,
/// is told.
fn after_1_26_50(payload: &mut Payload, cx: &mut UpgradeCx<'_>) {
    let mut told = Told::default();
    // The document's own `description.identifier`.
    let identifier = payload
        .description
        .as_object()
        .and_then(|description| description.get("identifier"))
        .and_then(Value::as_string)
        .map(|name| name.as_str().to_owned())
        .unwrap_or_default();
    let components = components(payload);
    if is_lightweight(components.get("minecraft:type_family")) {
        components.insert_absent(CAN_STAND_ON_POWDER_SNOW, VObject::new().into());
    }
    if let Some(had) = components.get(USES_LEGACY_AMBIENT_SOUNDS)
        && had.as_object().is_none_or(|members| !members.is_empty())
        && !had.is_null()
    {
        told.dropped(JsonPath::new().with(&["components", USES_LEGACY_AMBIENT_SOUNDS]));
    }
    components.set(USES_LEGACY_AMBIENT_SOUNDS, VObject::new().into());
    if identifier == "minecraft:spider" || identifier == "minecraft:cave_spider" {
        let mut immunity = VObject::new();
        let mut blocks = facet_value::VArray::new();
        blocks.push("minecraft:web");
        immunity.insert("blocks", blocks);
        components.insert_absent(SLOWDOWN_IMMUNITY, immunity.into());
    }
    if FREEZING_IMMUNE_MOBS.contains(&identifier.as_str()) {
        components.insert_absent(FREEZING_IMMUNE, VObject::new().into());
    }
    if FREEZING_VULNERABLE_MOBS.contains(&identifier.as_str()) {
        components.insert_absent(FREEZING_VULNERABLE, VObject::new().into());
    }
    if identifier == "minecraft:sulfur_cube"
        && let Some(group) = group(payload, SULFUR_CUBE_GROUP)
    {
        group.insert_absent(FREEZING_IMMUNE, VObject::new().into());
    }
    for (_, group) in payload.component_groups.iter_mut().flatten() {
        if let Some(group) = group.as_object_mut()
            && is_lightweight(group.get("minecraft:type_family"))
        {
            group.insert_absent(CAN_STAND_ON_POWDER_SNOW, VObject::new().into());
        }
    }
    told.report(cx);
}

/// The game's family test for `lightweight` on a
/// `minecraft:type_family` as written: its `family` list names `lightweight`.
fn is_lightweight(type_family: Option<&Value>) -> bool {
    type_family
        .and_then(Value::as_object)
        .and_then(|family| family.get("family"))
        .is_some_and(|family| match family.as_array() {
            Some(list) => list.iter().any(|name| {
                name.as_string()
                    .is_some_and(|n| n.as_str() == "lightweight")
            }),
            None => family
                .as_string()
                .is_some_and(|n| n.as_str() == "lightweight"),
        })
}

/// The legacy fixer's section A for `minecraft:enderman` below 1.26.50: upgrade of
/// `minecraft:teleport` gives each `teleport` `teleports_on_projectile_hit: true` and the
/// projectile filter, each when absent.
fn enderman_teleport(payload: &mut Payload) {
    for section in sections(payload) {
        if let Some(teleport) = section.members.object("minecraft:teleport") {
            teleport.insert_absent("teleports_on_projectile_hit", true.into());
            teleport.insert_absent("projectile_filter", fragments::enderman_projectile_filter());
        }
    }
}

/// The fixer's section B below 1.26.50: `projectile.multiple_targets: true` and
/// `behavior.follow_mob.abort_navigation_on_stop: false`, each when absent.
fn components_1_26_50(payload: &mut Payload) {
    for section in sections(payload) {
        if let Some(projectile) = section.members.object("minecraft:projectile") {
            projectile.insert_absent("multiple_targets", true.into());
        }
        if let Some(goal) = section.members.object("minecraft:behavior.follow_mob") {
            goal.insert_absent("abort_navigation_on_stop", false.into());
        }
    }
}
