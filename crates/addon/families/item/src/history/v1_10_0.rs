//! `v1_10_0`: the legacy item format of 1.10, and every document the game's
//! component-based check rejects (below 1.16.100, 1.16.101–1.16.x, and later documents
//! whose `components` name no registered component).

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::load::BuiltIns;
use bedrock_addon_core::schema;
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::{
    ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned, parse_payload, write_payload,
};
use facet::Facet;
use facet_value::{VArray, VObject};
use indexmap::IndexMap;

use super::document::Description;
use crate::codes;
use crate::legacy::{
    ItemFormat, LegacyCamera, LegacyEffect, LegacyFood, LegacyFormatItem, LegacyItemData,
    LegacySeed, number, seed_warning,
};
use crate::read::is_component_based;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath};

/// An item in the legacy item format (no `SemVersion`
/// reaches its readers, so one shape serves 1.10 to 1.16.0).
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    /// JSON `description`: `identifier` and the client's `category`.
    pub description: Description,
    /// JSON `components`: the fixed legacy key set.
    pub components: LegacyFormatItem,
    /// JSON `events`, as written.
    pub events: Option<Value>,
}

impl Versioned for Item {
    const VERSION: FormatVersion = known::V1_10_0;
}

/// What the legacy item is parsed from.
#[derive(Facet)]
#[facet(schema::unknown = "keep")]
struct Shell {
    description: Description,
    components: Option<Value>,
    events: Option<Value>,
}

impl VersionStruct for Item {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut found = Diagnostics::default();
        let shell: Option<Shell> = parse_payload(payload, cx, &mut found);
        // unknown keys are ignored silently at every version (and kept)
        // By legacy name: the variant and the definitions that were it.
        found.retain(|f| !matches!(f.code.name(), "UnrecognizedField"));
        d.extend_with_unknown(found);
        let mut shell = shell?;
        if let Err(reason) = is_component_based(payload, cx.written) {
            d.push(Diagnostic::of(core_codes::LEGACY_ITEM_FORMAT.with([reason])).at(cx.at.clone()));
        }
        if let Some(menu) = shell.description.menu_category.take()
            && let Ok(value) = write_payload(&menu)
        {
            d.unknown_mut()
                .put(&cx.at.child("description"), "menu_category", value);
        }
        let at = cx.at.child("components");
        let components = match &shell.components {
            None => LegacyFormatItem::default(),
            Some(value) => {
                let Some(object) = value.as_object() else {
                    d.push(Diagnostic::of(&codes::COMPONENTS_NOT_OBJECT).at(at));
                    return None;
                };
                read_format_item(
                    object,
                    &shell.description.identifier,
                    cx.load.built_ins(),
                    &at,
                    d,
                )
            }
        };
        if components.seed.is_some() && cx.original_version > known::V1_16_100 {
            seed_warning(&at, d);
        }
        Some(Item {
            description: shell.description,
            components,
            events: shell.events,
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        let mut object = VObject::new();
        object.insert("description", write_payload(&self.description)?);
        object.insert("components", format_item_value(&self.components));
        if let Some(events) = &self.events {
            object.insert("events", events.clone());
        }
        // The unknown members are the document's, written from its table.
        Ok(Value::from(object))
    }
}

// The 1.10.0 item holds no Molang a document's walk visits.
bedrock_addon_core::impl_no_molang!(Item);

/// An information at `at`: the member reads its value as another one than the document writes
/// (the legacy readers take any JSON value, as `asInt`, `asFloat`, `asBool` do; what they
/// make of it is said). The game is silent, so the finding is the crate's own information.
fn told(code: Code, at: JsonPath, d: &mut Diagnostics) {
    d.push(Diagnostic::info(code).at(at));
}

/// A value of another JSON type than the member's: an `InvalidType` information at `at`.
fn wrong_type(at: JsonPath, d: &mut Diagnostics) {
    told(Code::from(&core_codes::INVALID_TYPE), at, d);
}

/// A JSON number as the double it spells.
fn double(value: &Value) -> Option<f64> {
    value.as_number().map(facet_value::VNumber::to_f64_lossy)
}

/// `asInt`: a number truncated toward zero, a bool as 0 or 1. A fraction is truncated and a
/// number beyond an `i32` is not taken, each with an `InvalidValue` information at `at`; a
/// bool and any other value come with an `InvalidType` information.
#[allow(clippy::cast_possible_truncation, reason = "vanilla's asInt truncates")]
fn as_int(value: &Value, at: JsonPath, d: &mut Diagnostics) -> Option<i32> {
    let Some(number) = double(value) else {
        wrong_type(at, d);
        return value.as_bool().map(i32::from);
    };
    let whole = number.trunc();
    if whole < f64::from(i32::MIN) || whole > f64::from(i32::MAX) {
        told(Code::from(&core_codes::INVALID_VALUE), at, d);
        return None;
    }
    if number.fract() != 0.0 {
        told(Code::from(&core_codes::INVALID_VALUE), at, d);
    }
    Some(whole as i32)
}

/// `asFloat`. A number beyond an `f32` is the largest finite float of its sign, with the
/// information of such a number at `at`; another value comes with an `InvalidType`
/// information.
#[allow(clippy::cast_possible_truncation, reason = "an f32 member")]
fn as_float(value: &Value, at: JsonPath, d: &mut Diagnostics) -> Option<f32> {
    let Some(number) = double(value) else {
        wrong_type(at, d);
        return None;
    };
    let nearest = number as f32;
    if nearest.is_infinite() {
        d.push(bedrock_addon_core::molang::beyond_f32_warning(at, None));
        return Some(f32::MAX.copysign(nearest));
    }
    Some(nearest)
}

/// `asBool`: a bool, or a non-zero number (with an `InvalidType` information at `at`, as any
/// other value).
fn as_bool(value: &Value, at: JsonPath, d: &mut Diagnostics) -> Option<bool> {
    if let Some(flag) = value.as_bool() {
        return Some(flag);
    }
    wrong_type(at, d);
    number(value).map(|n| n != 0.0)
}

/// A string member; another value is not taken, with an `InvalidType` information at `at`.
fn as_string(value: &Value, at: JsonPath, d: &mut Diagnostics) -> Option<String> {
    let text = value.as_string().map(|s| s.as_str().to_owned());
    if text.is_none() {
        wrong_type(at, d);
    }
    text
}

/// The client keys of the legacy format, kept as written.
const CLIENT_KEYS: [&str; 6] = [
    "minecraft:icon",
    "minecraft:icon_variant",
    "minecraft:icon_atlas",
    "minecraft:use_animation",
    "minecraft:hover_text_color",
    "minecraft:rarity",
];

/// Reads the legacy key set from `components` (at `at`).
/// `effects` answers which mob effects the game has (the context's provider).
pub(crate) fn read_format_item(
    components: &VObject,
    identifier: &str,
    effects: &dyn BuiltIns,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> LegacyFormatItem {
    let mut item = LegacyFormatItem::default();
    for (key, value) in components {
        let key = key.as_str();
        match key {
            "minecraft:max_damage" => {
                item.max_damage = as_int(value, at.child(key), d).unwrap_or_default();
            }
            "minecraft:hand_equipped" => {
                item.hand_equipped = as_bool(value, at.child(key), d).unwrap_or_default();
            }
            "minecraft:stacked_by_data" => {
                item.stacked_by_data = as_bool(value, at.child(key), d).unwrap_or_default();
            }
            "minecraft:foil" => item.foil = as_bool(value, at.child(key), d).unwrap_or_default(),
            "minecraft:block" => item.block = as_string(value, at.child(key), d),
            "minecraft:use_duration" => {
                item.use_duration = as_int(value, at.child(key), d).unwrap_or_default();
            }
            "minecraft:max_stack_size" => {
                item.max_stack_size = as_int(value, at.child(key), d).unwrap_or(64);
            }
            "minecraft:food" => {
                item.food = Some(read_food(value, identifier, effects, &at.child(key), d));
            }
            "minecraft:seed" => item.seed = Some(read_seed(value, &at.child(key), d)),
            "minecraft:camera" => item.camera = Some(read_camera(value, &at.child(key), d)),
            other if CLIENT_KEYS.contains(&other) => {
                item.client.insert(other.to_owned(), value.clone());
            }
            other => {
                item.extra.insert(other.to_owned(), value.clone());
            }
        }
    }
    item
}

/// Whether `name` is one of the game's mob effects by `effects` (bare, without ASCII case);
/// a name it does not know is told by the caller.
fn known_effect(name: &str, effects: &dyn BuiltIns) -> bool {
    effects.effect(name)
}

/// `minecraft:food` of the legacy format as the game's legacy reader reads it.
fn read_food(
    value: &Value,
    identifier: &str,
    effects: &dyn BuiltIns,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> LegacyFood {
    let mut food = LegacyFood::default();
    let Some(object) = value.as_object() else {
        wrong_type(at.clone(), d);
        return food;
    };
    for (key, member) in object {
        let key = key.as_str();
        match key {
            "nutrition" => food.nutrition = as_int(member, at.child(key), d).unwrap_or_default(),
            "saturation_modifier" => food.saturation_modifier = Some(member.clone()),
            "using_converts_to" => food.using_converts_to = as_string(member, at.child(key), d),
            "on_use_action" => food.on_use_action = as_string(member, at.child(key), d),
            "on_use_range" => food.on_use_range = Some(member.clone()),
            "cooldown_type" => food.cooldown_type = as_string(member, at.child(key), d),
            "cooldown_time" => {
                food.cooldown_time = as_int(member, at.child(key), d).unwrap_or_default();
            }
            "can_always_eat" => {
                food.can_always_eat = as_bool(member, at.child(key), d).unwrap_or_default();
            }
            "effects" => {
                if !member.is_array() {
                    wrong_type(at.child(key), d);
                }
                for (i, effect) in member.as_array().into_iter().flatten().enumerate() {
                    let path = at.child("effects").index(i);
                    let Some(effect) = effect.as_object() else {
                        wrong_type(path, d);
                        continue;
                    };
                    // A member of the effect, read by `read` when it is written.
                    let name = effect
                        .get("name")
                        .and_then(|name| as_string(name, path.child("name"), d))
                        .unwrap_or_default();
                    if !known_effect(&name, effects) {
                        // The game logs an unknown effect name at the content log's error
                        // level and adds the effect on: a kept error.
                        d.push_kept_error(
                            Diagnostic::error(codes::LEGACY_EFFECT_NAME.with([
                                identifier,
                                "effect",
                                name.as_str(),
                            ]))
                            .at(path.clone()),
                        );
                    }
                    let chance = effect
                        .get("chance")
                        .and_then(|chance| as_float(chance, path.child("chance"), d));
                    let duration = effect
                        .get("duration")
                        .and_then(|duration| as_int(duration, path.child("duration"), d));
                    let amplifier = effect
                        .get("amplifier")
                        .and_then(|amplifier| as_int(amplifier, path.child("amplifier"), d));
                    food.effects.push(LegacyEffect {
                        name,
                        chance: chance.unwrap_or(1.0),
                        duration: duration.unwrap_or_default(),
                        amplifier: amplifier.unwrap_or_default(),
                    });
                }
            }
            "remove_effects" => {
                if !member.is_array() {
                    wrong_type(at.child(key), d);
                }
                for (i, name) in member.as_array().into_iter().flatten().enumerate() {
                    let path = at.child("remove_effects").index(i);
                    let name = as_string(name, path.clone(), d).unwrap_or_default();
                    if !known_effect(&name, effects) {
                        // The game logs an unknown name in `remove_effects` at the error
                        // level and the loop goes on: a kept error.
                        d.push_kept_error(
                            Diagnostic::error(codes::LEGACY_EFFECT_NAME.with([
                                identifier,
                                "remove_effect",
                                name.as_str(),
                            ]))
                            .at(path),
                        );
                    }
                    food.remove_effects.push(name);
                }
            }
            other => {
                food.extra.insert(other.to_owned(), member.clone());
            }
        }
    }
    food
}

/// `minecraft:seed` of the legacy format as the game's legacy reader reads it.
fn read_seed(value: &Value, at: &JsonPath, d: &mut Diagnostics) -> LegacySeed {
    let mut seed = LegacySeed::default();
    if !value.is_object() {
        wrong_type(at.clone(), d);
    }
    for (key, member) in value.as_object().into_iter().flat_map(VObject::iter) {
        let key = key.as_str();
        match key {
            "crop_result" => seed.crop_result = as_string(member, at.child(key), d),
            "plant_at" => seed.plant_at = Some(member.clone()),
            "plant_at_any_solid_surface" => {
                seed.plant_at_any_solid_surface =
                    as_bool(member, at.child(key), d).unwrap_or_default();
            }
            "plant_at_face" => seed.plant_at_face = as_string(member, at.child(key), d),
            other => {
                seed.extra.insert(other.to_owned(), member.clone());
            }
        }
    }
    seed
}

/// `minecraft:camera` of the legacy format as the game's legacy reader reads it.
fn read_camera(value: &Value, at: &JsonPath, d: &mut Diagnostics) -> LegacyCamera {
    let mut camera = LegacyCamera::default();
    if !value.is_object() {
        wrong_type(at.clone(), d);
    }
    for (key, member) in value.as_object().into_iter().flat_map(VObject::iter) {
        let key = key.as_str();
        let n = if matches!(
            key,
            "black_bars_duration"
                | "black_bars_screen_ratio"
                | "shutter_duration"
                | "shutter_screen_ratio"
                | "picture_duration"
                | "slide_away_duration"
        ) {
            as_float(member, at.child(key), d).unwrap_or_default()
        } else {
            0.0
        };
        match key {
            "black_bars_duration" => camera.black_bars_duration = n,
            "black_bars_screen_ratio" => camera.black_bars_screen_ratio = n,
            "shutter_duration" => camera.shutter_duration = n,
            "shutter_screen_ratio" => camera.shutter_screen_ratio = n,
            "picture_duration" => camera.picture_duration = n,
            "slide_away_duration" => camera.slide_away_duration = n,
            other => {
                camera.extra.insert(other.to_owned(), member.clone());
            }
        }
    }
    camera
}

/// A real number as JSON.
fn real(n: f32) -> Value {
    Value::from(facet_value::VNumber::from_f64(f64::from(n)))
}

/// An integer as JSON.
fn int(n: i32) -> Value {
    Value::from(facet_value::VNumber::from_i64(i64::from(n)))
}

/// The legacy key set as JSON: the members that differ from their defaults, then the client
/// keys and the unknown ones.
pub(crate) fn format_item_value(item: &LegacyFormatItem) -> Value {
    let mut object = VObject::new();
    if item.max_damage != 0 {
        object.insert("minecraft:max_damage", int(item.max_damage));
    }
    if item.hand_equipped {
        object.insert("minecraft:hand_equipped", Value::from(true));
    }
    if item.stacked_by_data {
        object.insert("minecraft:stacked_by_data", Value::from(true));
    }
    if item.foil {
        object.insert("minecraft:foil", Value::from(true));
    }
    if let Some(block) = &item.block {
        object.insert("minecraft:block", Value::from(block.as_str()));
    }
    if item.use_duration != 0 {
        object.insert("minecraft:use_duration", int(item.use_duration));
    }
    if item.max_stack_size != 64 {
        object.insert("minecraft:max_stack_size", int(item.max_stack_size));
    }
    if let Some(food) = &item.food {
        object.insert("minecraft:food", food_value(food));
    }
    if let Some(seed) = &item.seed {
        let mut out = VObject::new();
        if let Some(crop) = &seed.crop_result {
            out.insert("crop_result", Value::from(crop.as_str()));
        }
        if let Some(at) = &seed.plant_at {
            out.insert("plant_at", at.clone());
        }
        if seed.plant_at_any_solid_surface {
            out.insert("plant_at_any_solid_surface", Value::from(true));
        }
        if let Some(face) = &seed.plant_at_face {
            out.insert("plant_at_face", Value::from(face.as_str()));
        }
        for (key, value) in seed.extra.iter() {
            out.insert(key, value.clone());
        }
        object.insert("minecraft:seed", Value::from(out));
    }
    if let Some(camera) = &item.camera {
        let mut out = VObject::new();
        for (key, n) in [
            ("black_bars_duration", camera.black_bars_duration),
            ("black_bars_screen_ratio", camera.black_bars_screen_ratio),
            ("shutter_duration", camera.shutter_duration),
            ("shutter_screen_ratio", camera.shutter_screen_ratio),
            ("picture_duration", camera.picture_duration),
            ("slide_away_duration", camera.slide_away_duration),
        ] {
            out.insert(key, real(n));
        }
        for (key, value) in camera.extra.iter() {
            out.insert(key, value.clone());
        }
        object.insert("minecraft:camera", Value::from(out));
    }
    for (key, value) in &item.client {
        object.insert(key, value.clone());
    }
    for (key, value) in item.extra.iter() {
        object.insert(key, value.clone());
    }
    Value::from(object)
}

/// A legacy food as JSON.
fn food_value(food: &LegacyFood) -> Value {
    let mut out = VObject::new();
    out.insert("nutrition", int(food.nutrition));
    if let Some(saturation) = &food.saturation_modifier {
        out.insert("saturation_modifier", saturation.clone());
    }
    if let Some(to) = &food.using_converts_to {
        out.insert("using_converts_to", Value::from(to.as_str()));
    }
    if let Some(action) = &food.on_use_action {
        out.insert("on_use_action", Value::from(action.as_str()));
    }
    if let Some(range) = &food.on_use_range {
        out.insert("on_use_range", range.clone());
    }
    if let Some(kind) = &food.cooldown_type {
        out.insert("cooldown_type", Value::from(kind.as_str()));
    }
    if food.cooldown_time != 0 {
        out.insert("cooldown_time", int(food.cooldown_time));
    }
    if food.can_always_eat {
        out.insert("can_always_eat", Value::from(true));
    }
    if !food.effects.is_empty() {
        let mut effects = VArray::new();
        for effect in &food.effects {
            let mut e = VObject::new();
            e.insert("name", Value::from(effect.name.as_str()));
            e.insert("chance", real(effect.chance));
            e.insert("duration", int(effect.duration));
            e.insert("amplifier", int(effect.amplifier));
            effects.push(Value::from(e));
        }
        out.insert("effects", Value::from(effects));
    }
    if !food.remove_effects.is_empty() {
        let mut names = VArray::new();
        for name in &food.remove_effects {
            names.push(Value::from(name.as_str()));
        }
        out.insert("remove_effects", Value::from(names));
    }
    for (key, value) in food.extra.iter() {
        out.insert(key, value.clone());
    }
    Value::from(out)
}

/// the legacy reading is carried unchanged, not converted: `legacy.format` is
/// `Legacy`, `legacy.format_item` holds the key set, the component map is empty, and the
/// description's `category` moves to `legacy.category` (no `menu_category` is created).
impl Upgrade for Item {
    type Next = super::v1_16_100::Item;
    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Self::Next {
        let mut description = self.description;
        let category = description.category.take();
        let legacy = LegacyItemData {
            format: ItemFormat::Legacy,
            format_item: Some(self.components),
            category,
            ..LegacyItemData::default()
        };
        super::v1_16_100::Item {
            description,
            components: IndexMap::new(),
            deprecated: crate::ItemDeprecatedData::default(),
            events: self.events,
            legacy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::diagnostic::Severity;

    /// A provider that knows the effects of its list, compared as `known_effect` compared
    /// before the provider: bare and without ASCII case.
    struct Effects(&'static [&'static str]);

    impl BuiltIns for Effects {
        fn contains(&self, _kind: &'static str, _id: &str) -> bool {
            false
        }
        fn effect(&self, name: &str) -> bool {
            self.0.iter().any(|e| {
                e.trim_start_matches("minecraft:")
                    .eq_ignore_ascii_case(name)
            })
        }
    }

    /// an unknown `effects` or `remove_effects` name is a warning with vanilla's
    /// text, the effect kept.
    #[test]
    fn rule_19d_unknown_legacy_effect_names_warn() {
        let json: Value = bedrock_addon_core::json::from_str(
            r#"{"minecraft:food": {"nutrition": 1, "effects": [{"name": "speed"}, {"name": "zoom"}], "remove_effects": ["poison", "fizz"]}}"#,
        )
        .unwrap();
        let mut d = Diagnostics::default();
        let effects = Effects(&["minecraft:speed", "minecraft:poison"]);
        let item = read_format_item(
            json.as_object().unwrap(),
            "demo:x",
            &effects,
            &JsonPath::new(),
            &mut d,
        );
        let texts: Vec<String> = d.iter().map(|f| f.code.to_string()).collect();
        assert_eq!(
            texts,
            [
                "Item[demo:x] Failed to load effect with name[zoom] from Json. bad name?",
                "Item[demo:x] Failed to load remove_effect with name[fizz] from Json. bad name?",
            ]
        );
        // The game logs both at the error level and adds the effect on: kept errors.
        assert!(
            d.iter()
                .all(|f| (f.severity, f.kept) == (Severity::Error, true))
        );
        let food = item.food.unwrap();
        assert_eq!(food.effects.len(), 2);
        assert_eq!(food.remove_effects.len(), 2);
        // a provider that knows every name tells none
        let mut d = Diagnostics::default();
        read_format_item(
            json.as_object().unwrap(),
            "demo:x",
            &Effects(&["speed", "zoom", "poison", "fizz"]),
            &JsonPath::new(),
            &mut d,
        );
        assert!(d.is_empty());
    }
}
