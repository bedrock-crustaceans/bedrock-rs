//! Components about the entity's health and hurt: `boss`, `burns_in_daylight`,
//! `damage_over_time`, `exhaustion_values`, `insomnia`, `ignore_cannot_be_attacked`; and the
//! components 1.26.60 types first: `attack`, `spell_effects`, `strength`, `healable`.
// Types and defaults follow the game's 1.26.50 schemas (1.21.130 for `burns_in_daylight`,
// 1.26.40 for `ignore_cannot_be_attacked`).

use crate::component::descriptor_tags;
use bedrock_addon_core::diagnostic::{DiagnosticCode, JsonPath, VanillaText};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::shared::{Filter, FloatRange};
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_shared_shapes::ItemDescriptor;
use facet::Facet;
use facet_value::VObject;

use super::parse::{ComponentCheck, Typed};
use super::value::{EffectSeconds, EquipmentSlot, SecondsOrText, one_element};

/// `minecraft:boss`: the boss bar shown to players near the entity.
#[doc(alias = "minecraft:boss")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Boss {
    /// `name`: the name shown above the health bar; default empty.
    #[facet(default = String::new())]
    pub name: String,
    /// `should_darken_sky`: the sky darkens near the boss; default `false`.
    #[facet(default = false)]
    pub should_darken_sky: bool,
    /// `hud_range`: the distance in blocks within which the bar shows; default 55.
    #[facet(default = 55_i32)]
    pub hud_range: i32,
}

impl Typed for Boss {}

/// `minecraft:burns_in_daylight`: the entity burns in daylight unless the item in its
/// protection slot shields it.
#[doc(alias = "minecraft:burns_in_daylight")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct BurnsInDaylight {
    /// `protection_slot`: the slot whose item protects the entity; default `slot.armor.head`.
    #[facet(default = EquipmentSlot::ArmorHead)]
    pub protection_slot: EquipmentSlot,
}

impl Typed for BurnsInDaylight {}

/// `minecraft:damage_over_time`: the entity takes damage at a fixed interval.
#[doc(alias = "minecraft:damage_over_time")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DamageOverTime {
    /// `damage_per_hurt`: the damage of each hurt; default 1.
    #[facet(default = 1_i32)]
    pub damage_per_hurt: i32,
    /// `time_between_hurt`: the seconds between hurts; default 0.
    #[facet(default = 0.0_f32)]
    pub time_between_hurt: f32,
}

impl Typed for DamageOverTime {
    /// The legacy reader's camel-case spellings (`damagePerHurt|damage_per_hurt`,
    /// `timeBetweenHurt|time_between_hurt`).
    fn prepare(json: &mut VObject, cx: &mut ComponentCheck<'_>) -> bool {
        super::parse::legacy_spellings(
            json,
            &JsonPath::new(),
            &[
                ("damagePerHurt", "damage_per_hurt"),
                ("timeBetweenHurt", "time_between_hurt"),
            ],
            cx,
        )
    }
}

/// `minecraft:exhaustion_values`: how much exhaustion each player action adds.
#[doc(alias = "minecraft:exhaustion_values")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct ExhaustionValues {
    /// `attack`: per attack; default 0.1.
    #[facet(default = 0.1_f32)]
    pub attack: f32,
    /// `damage`: per damage taken; default 0.1.
    #[facet(default = 0.1_f32)]
    pub damage: f32,
    /// `heal`: per health point regenerated; default 6.
    #[facet(default = 6.0_f32)]
    pub heal: f32,
    /// `jump`: per jump; default 0.05.
    #[facet(default = 0.05_f32)]
    pub jump: f32,
    /// `lunge`: per lunge; default 4.
    #[facet(default = 4.0_f32)]
    pub lunge: f32,
    /// `mine`: per block mined; default 0.005.
    #[facet(default = 0.005_f32)]
    pub mine: f32,
    /// `sprint`: per metre sprinted; default 0.01.
    #[facet(default = 0.01_f32)]
    pub sprint: f32,
    /// `sprint_jump`: per sprinting jump; default 0.2.
    #[facet(default = 0.2_f32)]
    pub sprint_jump: f32,
    /// `swim`: per metre swum; default 0.01.
    #[facet(default = 0.01_f32)]
    pub swim: f32,
    /// `walk`: per metre walked; default 0.
    #[facet(default = 0.0_f32)]
    pub walk: f32,
}

impl Typed for ExhaustionValues {
    /// The lenient conversion reads a string (vanilla's player file writes `""`) as every member
    /// at its default, as the legacy parser does.
    fn shorthand(json: &Value, mode: ConvertMode) -> Option<VObject> {
        (matches!(mode, ConvertMode::Lenient) && json.is_string()).then(VObject::new)
    }
}

/// `minecraft:insomnia`: counts the days since the entity last rested, for phantom spawning.
#[doc(alias = "minecraft:insomnia")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Insomnia {
    /// `days_until_insomnia`: the days without rest before insomnia sets in; default 3.
    #[facet(default = 3.0_f32)]
    pub days_until_insomnia: f32,
}

impl Typed for Insomnia {
    /// At the typed version a `days_until_insomnia` that is no number is refused with the
    /// game's text `days_until_insomnia: invalid numeric value`.
    fn prepare(json: &mut VObject, cx: &mut ComponentCheck<'_>) -> bool {
        const MEMBER: &str = "days_until_insomnia";
        if matches!(cx.mode(), ConvertMode::Strict)
            && json
                .get(MEMBER)
                .is_some_and(|value| !value.is_null() && value.as_number().is_none())
        {
            cx.refuse(
                DiagnosticCode::InvalidType {
                    text: Some(VanillaText::Formatted {
                        template: "{}: {}",
                        args: vec![
                            MEMBER.to_owned(),
                            bedrock_addon_core::diagnostic::cereal::INVALID_NUMERIC_VALUE_TEXT
                                .to_owned(),
                        ],
                    }),
                },
                &JsonPath::new().child(MEMBER),
            );
        }
        false
    }
}

/// `minecraft:ignore_cannot_be_attacked`: the entity may attack entities that have
/// `minecraft:cannot_be_attacked`, those its filters pass (all of them without filters).
#[doc(alias = "minecraft:ignore_cannot_be_attacked")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct IgnoreCannotBeAttacked {
    /// `filters`: which protected entities may be attacked; absent means all.
    pub filters: Option<Filter>,
}

impl Typed for IgnoreCannotBeAttacked {
    fn check(&mut self, cx: &mut ComponentCheck<'_>) {
        if let Some(filters) = &self.filters {
            cx.extend(filters.check(&JsonPath::new().child("filters")));
        }
    }
}

/// `minecraft:attack`: the entity's melee attack and the effect it adds.
// `damage` is a nested `{min, max}` that also reads a plain number (default 0..0);
// `effect_amplifier` defaults to 0.
#[doc(alias = "minecraft:attack")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Attack {
    /// `damage`: the range of the damage dealt; a negative value heals. Default 0 to 0.
    #[facet(default = FloatRange::default())]
    pub damage: FloatRange,
    /// `effect_name`: the effect applied to the target; default none.
    #[facet(default = String::new())]
    pub effect_name: String,
    /// `effect_duration`: the effect's duration in seconds, or `infinite`; default 0.
    #[facet(default = EffectSeconds::default())]
    pub effect_duration: EffectSeconds,
    /// `effect_amplifier`: the effect's amplifier; default 0.
    #[facet(default = 0_i32)]
    pub effect_amplifier: i32,
}

impl Typed for Attack {
    fn prepare(json: &mut VObject, cx: &mut ComponentCheck<'_>) -> bool {
        matches!(cx.mode(), ConvertMode::Lenient)
            && super::movement::legacy_range(json, "damage", false, &JsonPath::new(), cx)
    }

    fn check(&mut self, cx: &mut ComponentCheck<'_>) {
        cx.range(&JsonPath::new().child("damage"), &mut self.damage);
    }
}

/// `minecraft:spell_effects`: effects added to and removed from the entity when the
/// component is added.
#[doc(alias = "minecraft:spell_effects")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SpellEffects {
    /// `add_effects`: the effects added.
    #[facet(default)]
    pub add_effects: Vec<SpellEffect>,
    /// `remove_effects`: the identifiers of the effects removed.
    #[facet(default)]
    pub remove_effects: Vec<String>,
}

/// One effect of [`SpellEffects`]; the
/// defaults are the typed reader's and the legacy parser's alike.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SpellEffect {
    /// `effect`: the effect's name; default none.
    #[facet(default = String::new())]
    pub effect: String,
    /// `duration`: the duration in seconds, or `infinite`; default 0.
    #[facet(default = EffectSeconds::default())]
    pub duration: EffectSeconds,
    /// `amplifier`: the effect's amplifier; default 0.
    #[facet(default = 0_i32)]
    pub amplifier: i32,
    /// `ambient`: the effect is ambient; default `false`.
    #[facet(default = false)]
    pub ambient: bool,
    /// `visible`: the effect is visible; default `true`.
    #[facet(default = true)]
    pub visible: bool,
    /// `display_on_screen_animation`: the effect shows its on-screen animation; default
    /// `false`.
    #[facet(default = false)]
    pub display_on_screen_animation: bool,
}

impl Typed for SpellEffects {
    const KEEP_NESTED: bool = true;

    /// Below 1.26.60 a single `remove_effects` name is the list of it (the legacy reader takes
    /// it); the typed reader refuses it: the game logs
    /// `expected an array` for a value that is no array.
    // The crate's own reading for the legacy side, taken from vanilla: the 1.15 `zombie_villager`
    // (1.26.0) writes `"remove_effects": "weakness"`, which the game loads.
    fn prepare(json: &mut VObject, cx: &mut ComponentCheck<'_>) -> bool {
        matches!(cx.mode(), ConvertMode::Lenient)
            && json.get("remove_effects").is_some_and(Value::is_string)
            && one_element(json, "remove_effects", &JsonPath::new(), cx)
    }
}

/// `minecraft:strength`: the entity's strength.
// `strength` defaults to 1 and `max` to 5.
#[doc(alias = "minecraft:strength")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Strength {
    /// `value`: the initial strength; default 1.
    #[facet(default = 1_i32)]
    pub value: i32,
    /// `max`: the most strength; default 5.
    #[facet(default = 5_i32)]
    pub max: i32,
}

impl Typed for Strength {}

/// `minecraft:healable`: the items that heal the entity.
// `healable` and its nested feed item and effect.
#[doc(alias = "minecraft:healable")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Healable {
    /// `items`: the items that heal it.
    #[facet(default)]
    pub items: Vec<HealItem>,
    /// `force_use`: an item is used even at full health; default `false`.
    #[facet(default = false)]
    pub force_use: bool,
    /// `filters`: when an item may be used; absent means always.
    pub filters: Option<Filter>,
}

/// One item of [`Healable`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct HealItem {
    /// `item`: the item; absent is no item.
    #[facet(schema::molang_with = descriptor_tags)]
    pub item: Option<ItemDescriptor>,
    /// `result_item`: the item the used one turns into; absent is none.
    #[facet(schema::molang_with = descriptor_tags)]
    pub result_item: Option<ItemDescriptor>,
    /// `heal_amount`: the health it gives; default 1.
    #[facet(default = 1_i32)]
    pub heal_amount: i32,
    /// `effects`: the effects it applies.
    #[facet(default)]
    pub effects: Vec<HealEffect>,
}

/// One effect of a [`HealItem`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct HealEffect {
    /// `name`: the effect's name; default none.
    #[facet(default = String::new())]
    pub name: String,
    /// `duration`: the duration in seconds, or a text (`"infinite"`); default 0.
    #[facet(default = SecondsOrText::default())]
    pub duration: SecondsOrText,
    /// `amplifier`: the effect's amplifier; default 1.
    #[facet(default = 1_i32)]
    pub amplifier: i32,
    /// `chance`: the chance the effect applies; default 1.
    #[facet(default = 1.0_f32)]
    pub chance: f32,
}

impl Typed for Healable {
    // Not KEEP_NESTED: the legacy reader is one of the game's schema readers, which validates
    // the whole tree before it parses: a nested value of another type never reaches this
    // read; from 1.15.0 it defaults the whole component.

    /// Below 1.26.60 the legacy schema takes an item written as its name alone and a single
    /// effect object; the typed reader refuses both shapes.
    fn prepare(json: &mut VObject, cx: &mut ComponentCheck<'_>) -> bool {
        if !matches!(cx.mode(), ConvertMode::Lenient) {
            return false;
        }
        let Some(items) = json.get_mut("items").and_then(Value::as_array_mut) else {
            return false;
        };
        let list = JsonPath::new().child("items");
        let mut changed = false;
        for (i, item) in items.iter_mut().enumerate() {
            let at = list.index(i);
            if item.is_string() {
                let name = std::mem::replace(item, Value::NULL);
                let mut object = VObject::new();
                object.insert("item", name);
                *item = object.into();
                cx.moved(at.clone(), at.child("item"));
                changed = true;
            }
            if let Some(object) = item.as_object_mut()
                && object.get("effects").is_some_and(Value::is_object)
            {
                changed |= one_element(object, "effects", &at, cx);
            }
        }
        changed
    }

    fn check(&mut self, cx: &mut ComponentCheck<'_>) {
        let list = JsonPath::new().child("items");
        for (i, item) in self.items.iter().enumerate() {
            let at = list.index(i);
            for (key, descriptor) in [("item", &item.item), ("result_item", &item.result_item)] {
                if let Some(descriptor) = descriptor {
                    cx.extend(descriptor.check(&at.child(key)));
                }
            }
        }
        if let Some(filters) = &self.filters {
            cx.extend(filters.check(&JsonPath::new().child("filters")));
        }
    }
}
