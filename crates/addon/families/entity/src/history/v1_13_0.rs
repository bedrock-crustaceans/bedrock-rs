//! Format 1.13.0: the struct and the step to 1.15.0.

super::legacy_struct!(1, 13, 0);

use bedrock_addon_core::diagnostic::{DiagnosticCode, JsonPath, VanillaText};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};
use facet_value::VObject;

use super::ops::{
    Members, Removed, Told, components, f32_value, finding, int_of, json, removed, section_a,
    section_b, sections,
};

/// The step's version, for its findings.
const V1_15_0: FormatVersion = FormatVersion::new(1, 15, 0);

/// 1.13.0 → 1.15.0 (gate 1.15.0).
impl Upgrade for Entity {
    type Next = super::v1_15_0::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        if section_a(&payload, cx) {
            removals(&mut payload, cx);
            per_type(&mut payload, cx);
            circle_around_anchor(&mut payload, cx);
            for goal in [
                "minecraft:behavior.stomp_turtle_egg",
                "minecraft:behavior.lay_egg",
            ] {
                remove_member(&mut payload, cx, goal, "search_count");
            }
        }
        if section_b(&payload, cx) {
            // The 1.15 spawn-entity rewrite: an array, or an object without `entities`, becomes
            // `{entities: <old>}`; the move is recorded, so what is told below it later (the
            // game's schema-reader lines) is told where the file has it.
            let mut told = Told::default();
            for section in sections(&mut payload) {
                if let Some(spawn) = section.members.get_mut("minecraft:spawn_entity") {
                    let wrap = spawn.is_array()
                        || spawn
                            .as_object()
                            .is_some_and(|o| !o.contains_key("entities"));
                    if wrap {
                        let mut wrapped = VObject::new();
                        wrapped.insert("entities", spawn.clone());
                        *spawn = wrapped.into();
                        let at = section.at.child("minecraft:spawn_entity");
                        told.moved(at.clone(), at.child("entities"));
                    }
                }
            }
            told.report(cx);
        }
        super::v1_15_0::Entity(payload)
    }
}

use ActorType as T;

/// Removes `member` of `component` in every section each removal a `DroppedField` warning.
fn remove_member(payload: &mut Payload, cx: &mut UpgradeCx<'_>, component: &str, member: &str) {
    let mut gone = Vec::new();
    for section in sections(payload) {
        if let Some(object) = section.members.object(component)
            && object.remove(member).is_some()
        {
            gone.push(section.at.with(&[component, member]));
        }
    }
    for at in gone {
        removed(cx, Removed::Field, &at, member, V1_15_0);
    }
}

/// The per-type removals in every section.
fn removals(payload: &mut Payload, cx: &mut UpgradeCx<'_>) {
    const MELEE: &str = "minecraft:behavior.melee_attack";
    let members: &[(&str, &str)] = match cx.actor_type {
        T::Llama | T::Horse | T::Donkey | T::Mule | T::ZombieHorse => {
            &[("minecraft:rideable", "priority")]
        }
        T::Blaze => &[(MELEE, "max_dist")],
        T::Bee => &[
            (MELEE, "look_distance"),
            (MELEE, "untrackable_cooldown_delay"),
            (MELEE, "target_tracking"),
            (MELEE, "target_dist"),
        ],
        T::IronGolem
        | T::Pillager
        | T::Panda
        | T::Fox
        | T::Wolf
        | T::Skeleton
        | T::Stray
        | T::WitherSkeleton => &[(MELEE, "target_dist")],
        _ => &[],
    };
    for (component, member) in members {
        remove_member(payload, cx, component, member);
    }
}

/// The per-type ops on `components` and the non-mob `damage_sensor` removal.
fn per_type(payload: &mut Payload, cx: &mut UpgradeCx<'_>) {
    let actor = cx.actor_type;
    let components = components(payload);
    if actor == T::Dolphin {
        components.insert_absent("minecraft:behavior.swim_with_entity", json("{}"));
    }
    if actor == T::Silverfish {
        let first = components
            .object("minecraft:behavior.nearest_attackable_target")
            .and_then(|goal| goal.get_mut("entity_types"))
            .and_then(|types| types.as_array_mut())
            .and_then(|types| types.get_mut(0))
            .and_then(|first| first.as_object_mut());
        if let Some(first) = first
            && first.remove("attack_interval").is_some()
        {
            let mut at = JsonPath::new().with(&[
                "components",
                "minecraft:behavior.nearest_attackable_target",
                "entity_types",
            ]);
            at.push_index(0);
            at.push("attack_interval");
            removed(cx, Removed::Field, &at, "attack_interval", V1_15_0);
        }
    }
    if matches!(actor, T::Pufferfish | T::Salmon | T::Tropicalfish | T::Cod)
        && let Some(wander) = components.object("minecraft:behavior.swim_wander")
    {
        // A `null` member is an absent one.
        if wander.get("interval").is_some_and(Value::is_null) {
            wander.remove("interval");
        }
        if let Some(interval) = wander.get_mut("interval") {
            // A 1-in-n tick chance becomes a probability (`f32` division, widened). A value that is
            // no int is read as vanilla's `asInt` reads it, and told at the member.
            let mut told = Told::default();
            let n = told.read(int_of(interval), || {
                JsonPath::new().with(&["components", "minecraft:behavior.swim_wander", "interval"])
            });
            #[allow(clippy::cast_precision_loss, reason = "vanilla's f32 division")]
            let chance = if n == 0 { 1.0 } else { 1.0 / n as f32 };
            *interval = f32_value(chance);
            told.report(cx);
        }
    }
    // Not a mob (the type id lacks the mob bit): `damage_sensor` cannot work below 1.15.
    if actor.id() & 0x100 == 0 {
        let mut gone = Vec::new();
        for section in sections(payload) {
            if section.members.take("minecraft:damage_sensor").is_some() {
                gone.push(section.at.child("minecraft:damage_sensor"));
            }
        }
        // One finding at every location the component is removed from.
        for at in gone {
            finding(
                cx,
                DiagnosticCode::DroppedComponent {
                    text: Some(VanillaText::Fixed(
                        "The component minecraft:damage_sensor will not be added to this entity as it won't work unless the format version is 1.15 or greater.",
                    )),
                },
                &at,
            );
        }
    }
}

/// `circle_around_anchor` (all types): when the goal is in `components`, each old chance member
/// is read as an int `n`, the new one written into the `components` goal only as `1.0 / n`
/// (`1.0` when `n` < 1), and the old one removed from every section. In a group the old member
/// is removed without a replacement, so each such removal is a `DroppedField` warning.
fn circle_around_anchor(payload: &mut Payload, cx: &mut UpgradeCx<'_>) {
    const GOAL: &str = "minecraft:behavior.circle_around_anchor";
    let pairs = [
        ("radius_change_chance", "radius_adjustment_chance"),
        ("height_change_chance", "height_adjustment_chance"),
    ];
    if !payload.components.as_ref().is_some_and(|c| c.has(GOAL)) {
        return;
    }
    // The crate's own note: a chance that is no int is read as vanilla's `asInt` reads it and told
    // at the member; a new member the document already has is replaced, which is told
    // (INFO: the fixer does it) when it held another value.
    let mut told = Told::default();
    for (old, new) in pairs {
        // A `null` member is an absent one.
        if let Some(goal) = components(payload).object(GOAL)
            && goal.get(old).is_some_and(Value::is_null)
        {
            goal.remove(old);
        }
        let value = components(payload)
            .object(GOAL)
            .and_then(|goal| goal.get(old).cloned());
        let Some(value) = value else {
            continue;
        };
        let n = told.read(int_of(&value), || {
            JsonPath::new().with(&["components", GOAL, old])
        });
        #[allow(clippy::cast_precision_loss, reason = "vanilla's f32 division")]
        let chance = if n < 1 { 1.0 } else { 1.0 / n as f32 };
        if let Some(goal) = components(payload).object(GOAL) {
            let chance = f32_value(chance);
            if goal
                .get(new)
                .is_some_and(|had| !had.is_null() && *had != chance)
            {
                told.dropped(JsonPath::new().with(&["components", GOAL, new]));
            }
            goal.insert(new, chance);
        }
        // `sections` lists `components` first; the goal is there (checked above).
        for (index, section) in sections(payload).into_iter().enumerate() {
            let at = section.at;
            if let Some(goal) = section.members.object(GOAL)
                && goal.remove(old).is_some()
                && index > 0
            {
                removed(cx, Removed::Field, &at.with(&[GOAL, old]), old, V1_15_0);
            }
        }
    }
    told.report(cx);
}
