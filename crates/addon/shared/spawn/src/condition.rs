//! The legacy spawn condition (`conditions[]` of 1.8.0-shaped spawn rules and of spawn groups):
//! its members as written, the game's hand-written reader (shared line for line by both
//! families) and the writer.

use facet_value::{VArray, VObject};

use super::read::{self, child, element, finding, wrong_type};
use super::texts;
use bedrock_addon_core::diagnostic::{
    DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::EntityId;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::shared::Filter;
use bedrock_addon_shared_shapes::BlockDescriptor;

/// One descriptor or a list of them, as written (shapes').
pub use bedrock_addon_shared_shapes::OneOrMany;

/// One entry of `conditions`: where, when and how an entity spawns (with the
/// spawn rules vanilla derives from it).
///
/// Every member is optional and named by its JSON key without `minecraft:`. A member that is
/// present but of a shape vanilla cannot read (a number for `minecraft:height_filter`, …) is
/// warned, kept as written in `extra` and read as vanilla reads it;
/// [`Condition::to_spawn_rules`] gives the values vanilla computes from the condition.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Condition {
    /// The condition's spawn weight (JSON `minecraft:weight`).
    pub weight: Option<Weight>,
    /// How many of the entity's pool may exist (JSON `minecraft:density_limit`).
    pub density_limit: Option<DensityLimit>,
    /// The blocks the entity may spawn on (JSON `minecraft:spawns_on_block_filter`).
    pub spawns_on_block_filter: Option<OneOrMany<BlockDescriptor>>,
    /// The blocks the entity may not spawn on (JSON
    /// `minecraft:spawns_on_block_prevented_filter`).
    pub spawns_on_block_prevented_filter: Option<OneOrMany<BlockDescriptor>>,
    /// Blocks that must be found above the spawn position (JSON
    /// `minecraft:spawns_above_block_filter`).
    pub spawns_above_block_filter: Option<AboveBlockFilter>,
    /// How many spawn at once, and the events they get (JSON `minecraft:herd`, one object or an
    /// array).
    pub herd: Option<OneOrMany<Herd>>,
    /// Other entities spawned in the entity's place (JSON `minecraft:permute_type`, one object
    /// or an array).
    pub permute_type: Option<OneOrMany<Permutation>>,
    /// The light level range (JSON `minecraft:brightness_filter`).
    pub brightness_filter: Option<BrightnessFilter>,
    /// The height range (JSON `minecraft:height_filter`).
    pub height_filter: Option<RangeFilter>,
    /// Spawns on the surface; vanilla tests only that the member exists, whatever its value
    /// (JSON `minecraft:spawns_on_surface`, the value as written).
    pub spawns_on_surface: Option<Value>,
    /// Spawns underground; present = set (JSON `minecraft:spawns_underground`).
    pub spawns_underground: Option<Value>,
    /// Spawns in water; present = set (JSON `minecraft:spawns_underwater`).
    pub spawns_underwater: Option<Value>,
    /// Spawns in lava; present = set (JSON `minecraft:spawns_lava`).
    pub spawns_lava: Option<Value>,
    /// Never spawns in a bubble column; present = set (JSON
    /// `minecraft:disallow_spawns_in_bubble`).
    pub disallow_spawns_in_bubble: Option<Value>,
    /// The biomes the condition applies in; absent = every biome (JSON `minecraft:biome_filter`).
    pub biome_filter: Option<Filter>,
    /// The world difficulty range (JSON `minecraft:difficulty_filter`).
    pub difficulty_filter: Option<DifficultyFilter>,
    /// The distance range from the nearest player (JSON `minecraft:distance_filter`).
    pub distance_filter: Option<RangeFilter>,
    /// Only with the experimental gameplay toggle; present = set (JSON
    /// `minecraft:is_experimental`).
    pub is_experimental: Option<Value>,
    /// The world age range in ticks (JSON `minecraft:world_age_filter`).
    pub world_age_filter: Option<WorldAgeFilter>,
    /// A delay before the spawn (JSON `minecraft:delay_filter`).
    pub delay_filter: Option<DelayFilter>,
    /// A mob event that must be on (JSON `minecraft:mob_event_filter`).
    pub mob_event_filter: Option<MobEventFilter>,
    /// Spawned entities never despawn; present = set (JSON `minecraft:is_persistent`).
    pub is_persistent: Option<Value>,
    /// Only near a village a player is in (JSON `minecraft:player_in_village_filter`).
    pub player_in_village_filter: Option<VillageFilter>,
    /// Kept as written; vanilla does not read it (JSON `minecraft:spawn_event`).
    pub spawn_event: Option<Value>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// `minecraft:weight`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Weight {
    /// The condition's weight; absent = 100, a negative value counts as 0 (JSON `default`).
    pub default: Option<i32>,
    /// The rarity; absent = 0, `null` = 5 (JSON `rarity`).
    pub rarity: Option<i32>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// `minecraft:density_limit`: population caps, −1 for none.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DensityLimit {
    /// The cap on the surface; absent = −1 (JSON `surface`).
    pub surface: Option<i32>,
    /// The cap underground; absent = −1 (JSON `underground`).
    pub underground: Option<i32>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// `minecraft:spawns_above_block_filter`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AboveBlockFilter {
    /// The blocks (JSON `blocks`).
    pub blocks: Option<OneOrMany<BlockDescriptor>>,
    /// How far above; absent = 1 (JSON `distance`).
    pub distance: Option<i32>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// One entry of `minecraft:herd` (one herd). No `min_size ≤ max_size` check;
/// a negative size wraps to a large count, as in vanilla.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Herd {
    /// The smallest herd; absent = 1 (JSON `min_size`).
    pub min_size: Option<i32>,
    /// The largest herd; absent = 1 (JSON `max_size`).
    pub max_size: Option<i32>,
    /// An entity event sent to the herd's members (JSON `event`).
    pub event: Option<String>,
    /// How many members skip `event`; absent = 0 (JSON `event_skip_count`).
    pub event_skip_count: Option<i32>,
    /// An entity event sent to the first members (JSON `initial_event`).
    pub initial_event: Option<String>,
    /// How many members get `initial_event`; absent = 0 (JSON `initial_event_count`).
    pub initial_event_count: Option<i32>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// One entry of `minecraft:permute_type`: an entity spawned instead of the condition's own.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Permutation {
    /// The weight in the weighted pick; absent = 1, below 1 leaves the entity out of it (JSON
    /// `weight`).
    pub weight: Option<i32>,
    /// How many are spawned regardless of the pick; absent = 0 (JSON `guaranteed_count`).
    pub guaranteed_count: Option<i32>,
    /// The entity (JSON `entity_type`).
    pub entity_type: Option<EntityId>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// `minecraft:brightness_filter`: the light level range, each end clamped to 0..=15.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BrightnessFilter {
    /// The lowest light level; absent = 0 (JSON `min`).
    pub min: Option<i32>,
    /// The highest light level; absent = 15 (JSON `max`).
    pub max: Option<i32>,
    /// Darken for weather (JSON `adjust_for_weather`).
    pub adjust_for_weather: Option<bool>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// `minecraft:height_filter` and `minecraft:distance_filter`: an integer range whose absent ends
/// are 24 and 128 (vanilla's default spawn distances, for both keys); `min > max` is swapped.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RangeFilter {
    /// The lower end; absent = 24 (JSON `min`).
    pub min: Option<i32>,
    /// The upper end; absent = 128 (JSON `max`).
    pub max: Option<i32>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// `minecraft:difficulty_filter`: names compared case-insensitively; an unknown name takes the
/// end's default.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DifficultyFilter {
    /// The lowest difficulty; absent or unknown = `peaceful` (JSON `min`).
    pub min: Option<String>,
    /// The highest difficulty; absent or unknown = `hard` (JSON `max`).
    pub max: Option<String>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// `minecraft:world_age_filter`: a range of world ages in ticks, compared unsigned (a negative
/// number is stored as its two's complement).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldAgeFilter {
    /// The lowest age; absent = 0 (JSON `min`).
    pub min: Option<u64>,
    /// The highest age; absent = `u64::MAX` (JSON `max`).
    pub max: Option<u64>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// `minecraft:delay_filter`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DelayFilter {
    /// The shortest delay; absent = 0, a negative one counts as 0 (JSON `min`).
    pub min: Option<i32>,
    /// The longest delay; absent = 0 (JSON `max`).
    pub max: Option<i32>,
    /// The entity the delay is for (JSON `identifier`).
    pub identifier: Option<EntityId>,
    /// The chance in percent; absent = 100 (JSON `spawn_chance`).
    pub spawn_chance: Option<i32>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// `minecraft:mob_event_filter`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MobEventFilter {
    /// The mob event's name (JSON `event`).
    pub event: Option<String>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// `minecraft:player_in_village_filter`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VillageFilter {
    /// The distance from the village; absent = 0 with a warning (JSON `distance`).
    pub distance: Option<u32>,
    /// How far outside the village border still counts; absent = 10 (JSON
    /// `village_border_tolerance`).
    pub village_border_tolerance: Option<u32>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// The JSON keys of a condition, in the order the game's reader tests them, then
/// `minecraft:spawn_event`, which it never reads.
pub const KEYS: &[&str] = &[
    "minecraft:weight",
    "minecraft:density_limit",
    "minecraft:spawns_on_block_filter",
    "minecraft:spawns_on_block_prevented_filter",
    "minecraft:spawns_above_block_filter",
    "minecraft:herd",
    "minecraft:permute_type",
    "minecraft:brightness_filter",
    "minecraft:height_filter",
    "minecraft:spawns_on_surface",
    "minecraft:spawns_underground",
    "minecraft:spawns_underwater",
    "minecraft:spawns_lava",
    "minecraft:disallow_spawns_in_bubble",
    "minecraft:biome_filter",
    "minecraft:difficulty_filter",
    "minecraft:distance_filter",
    "minecraft:is_experimental",
    "minecraft:world_age_filter",
    "minecraft:delay_filter",
    "minecraft:mob_event_filter",
    "minecraft:is_persistent",
    "minecraft:player_in_village_filter",
    "minecraft:spawn_event",
];

/// What differs between the two parsers that share the condition reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Profile {
    /// The cap `distance_filter.max` is clamped to: 128 for spawn rules, 1024 for spawn groups
    /// (the spawn-distance cap is 1024 before the keys are read).
    pub distance_cap: i32,
}

/// The spawn-rules parser.
pub const SPAWN_RULES: Profile = Profile { distance_cap: 128 };
/// The spawn-groups parser.
pub const SPAWN_GROUPS: Profile = Profile { distance_cap: 1024 };

/// The lower default of `height_filter` and `distance_filter`.
pub const MIN_DEFAULT_SPAWN_DISTANCE: i32 = 24;
/// The upper default of `height_filter` and `distance_filter`.
pub const MAX_DEFAULT_SPAWN_DISTANCE: i32 = 128;

/// What the condition reader needs besides the value.
#[derive(Debug)]
pub(crate) struct ReadCx<'a> {
    /// The document's identifier, for the texts that name it.
    pub identifier: &'a str,
    /// The parser's profile.
    pub profile: Profile,
    /// Where findings go.
    pub d: &'a mut Diagnostics,
}

impl ReadCx<'_> {
    /// A warning with vanilla's formatted text naming the identifier.
    fn formatted(
        &mut self,
        code: fn(Option<VanillaText>) -> DiagnosticCode,
        template: &'static str,
        at: &JsonPath,
    ) {
        let text = VanillaText::Formatted {
            template,
            args: vec![self.identifier.to_owned()],
        };
        // The legacy parsers log each condition text at code 2, level 2 in both masks.
        finding(code(Some(text)), Severity::Warning, at, self.d);
    }

    /// A warning with a fixed vanilla text.
    fn fixed(
        &mut self,
        code: fn(Option<VanillaText>) -> DiagnosticCode,
        text: &'static str,
        at: &JsonPath,
    ) {
        // Code 2, level 2 as above.
        finding(
            code(Some(VanillaText::Fixed(text))),
            Severity::Warning,
            at,
            self.d,
        );
    }

    /// The object-form warning of a key vanilla reads only from an object.
    fn object_form(&mut self, template: &'static str, at: &JsonPath) {
        self.formatted(|text| DiagnosticCode::InvalidType { text }, template, at);
    }
}

fn invalid_value(text: Option<VanillaText>) -> DiagnosticCode {
    DiagnosticCode::InvalidValue { text }
}

fn missing(text: Option<VanillaText>) -> DiagnosticCode {
    DiagnosticCode::MissingRequiredField { text }
}

/// The members of `object` other than `known`, each an `UnrecognizedField` warning (the
/// crate's own: vanilla's hand parser never looks at them), kept for write-back.
pub fn unknown_members(
    object: &VObject,
    known: &[&str],
    at: &JsonPath,
    d: &mut Diagnostics,
) -> UnknownMembers {
    members(object, known, &[], at, d)
}

/// [`unknown_members`], and the members `kept` (known members of a shape the reader could not
/// hold, already reported) copied without a finding, in document order: write-back never drops
/// or rewrites a member the author wrote.
pub fn members(
    object: &VObject,
    known: &[&str],
    kept: &[&str],
    at: &JsonPath,
    d: &mut Diagnostics,
) -> UnknownMembers {
    let mut extra = UnknownMembers::default();
    for (key, value) in object {
        if kept.contains(&key.as_str()) {
            extra.insert(key.as_str().to_owned(), value.clone());
        } else if !known.contains(&key.as_str()) {
            // The crate's own information: the legacy parsers read the keys they know and never
            // walk the members.
            finding(
                DiagnosticCode::UnrecognizedField { text: None },
                Severity::Info,
                &child(at, key.as_str()),
                d,
            );
            extra.insert(key.as_str().to_owned(), value.clone());
        }
    }
    extra
}

/// Reads one `conditions[]` element; a non-object element is an `InvalidType` warning and
/// skipped (this crate's choice; vanilla adds a default condition).
pub(crate) fn read_condition(
    value: &Value,
    at: &JsonPath,
    cx: &mut ReadCx<'_>,
) -> Option<Condition> {
    let Some(object) = value.as_object() else {
        wrong_type(at, None, cx.d);
        return None;
    };
    let get = |key: &str| object.get(key);
    let mut c = Condition::default();
    let path = |key: &str| child(at, key);
    // Keys present but of a shape the typed member cannot hold: kept as written in `extra`.
    let mut kept: Vec<&str> = Vec::new();
    macro_rules! member {
        ($index:literal, $field:ident, $read:expr) => {
            if let Some(v) = get(KEYS[$index]) {
                c.$field = $read(v, &path(KEYS[$index]));
                if c.$field.is_none() {
                    kept.push(KEYS[$index]);
                }
            }
        };
    }
    member!(0, weight, |v, at: &JsonPath| weight(v, at, cx));
    member!(1, density_limit, |v, at: &JsonPath| density_limit(
        v, at, cx
    ));
    member!(2, spawns_on_block_filter, |v, at: &JsonPath| blocks(
        v, at, cx.d
    ));
    member!(3, spawns_on_block_prevented_filter, |v, at: &JsonPath| {
        blocks(v, at, cx.d)
    });
    member!(
        4,
        spawns_above_block_filter,
        |v, at: &JsonPath| above_block(v, at, cx.d)
    );
    member!(5, herd, |v, at: &JsonPath| one_or_many(v, at, cx, herd));
    member!(6, permute_type, |v, at: &JsonPath| one_or_many(
        v,
        at,
        cx,
        permutation
    ));
    member!(7, brightness_filter, |v, at: &JsonPath| brightness(
        v, at, cx
    ));
    member!(8, height_filter, |v, at: &JsonPath| height(v, at, cx));
    // The flags: only the member's presence is tested, the value is never read.
    c.spawns_on_surface = get(KEYS[9]).cloned();
    c.spawns_underground = get(KEYS[10]).cloned();
    c.spawns_underwater = get(KEYS[11]).cloned();
    c.spawns_lava = get(KEYS[12]).cloned();
    c.disallow_spawns_in_bubble = get(KEYS[13]).cloned();
    member!(14, biome_filter, |v, at: &JsonPath| biome_filter(
        v, at, cx.d
    ));
    member!(15, difficulty_filter, |v, at: &JsonPath| difficulty(
        v, at, cx.d
    ));
    member!(16, distance_filter, |v, at: &JsonPath| distance(v, at, cx));
    c.is_experimental = get(KEYS[17]).cloned();
    member!(18, world_age_filter, |v, at: &JsonPath| world_age(
        v, at, cx
    ));
    member!(19, delay_filter, |v, at: &JsonPath| delay(v, at, cx));
    member!(20, mob_event_filter, |v, at: &JsonPath| mob_event(
        v, at, cx
    ));
    c.is_persistent = get(KEYS[21]).cloned();
    member!(22, player_in_village_filter, |v, at: &JsonPath| village(
        v, at, cx
    ));
    // Not read by vanilla; kept without a finding.
    c.spawn_event = get(KEYS[23]).cloned();
    c.extra = members(object, KEYS, &kept, at, cx.d);
    Some(c)
}

fn weight(value: &Value, at: &JsonPath, cx: &mut ReadCx<'_>) -> Option<Weight> {
    let Some(o) = value.as_object() else {
        cx.object_form(texts::WEIGHT_OBJECT, at);
        return None;
    };
    Some(Weight {
        default: read::int(o.get("default"), &child(at, "default"), cx.d),
        // Read only when the member exists, with a default of 5: a `null` is 5, not the 0 of
        // an absent member. The crate tells it at the member, since a `null` is written as
        // the 5 it is read as.
        rarity: o.contains_key("rarity").then(|| {
            let at = child(at, "rarity");
            read::int(o.get("rarity"), &at, cx.d).unwrap_or_else(|| {
                wrong_type(&at, None, cx.d);
                5
            })
        }),
        extra: unknown_members(o, &["default", "rarity"], at, cx.d),
    })
}

fn density_limit(value: &Value, at: &JsonPath, cx: &mut ReadCx<'_>) -> Option<DensityLimit> {
    let Some(o) = value.as_object() else {
        cx.object_form(texts::DENSITY_OBJECT, at);
        return None;
    };
    Some(DensityLimit {
        surface: read::int(o.get("surface"), &child(at, "surface"), cx.d),
        underground: read::int(o.get("underground"), &child(at, "underground"), cx.d),
        extra: unknown_members(o, &["surface", "underground"], at, cx.d),
    })
}

/// A block list: an empty value (`null`, `[]`, `{}`) adds
/// nothing, an array one descriptor per element, anything else one descriptor. A descriptor that
/// fails gets its `InvalidValue` warning and is dropped (this crate's choice; vanilla keeps an
/// empty descriptor). `None` (the caller keeps
/// the value as written) for `null`, `{}` and a single descriptor that fails.
fn blocks(value: &Value, at: &JsonPath, d: &mut Diagnostics) -> Option<OneOrMany<BlockDescriptor>> {
    if value.is_null() || value.as_object().is_some_and(VObject::is_empty) {
        return None;
    }
    if let Some(array) = value.as_array() {
        let mut list = Vec::with_capacity(array.len());
        let mut places = Vec::with_capacity(array.len());
        for (i, element) in array.iter().enumerate() {
            if let Some(read) = descriptor(element, &read::element(at, i), d) {
                list.push(read);
                places.push(i);
            }
        }
        // What is told of a descriptor from here on is at the index the file has.
        d.left_out(at, &places);
        return Some(OneOrMany::Many(list));
    }
    descriptor(value, at, d).map(OneOrMany::One)
}

/// One descriptor and its findings; `None` for a shape vanilla's reader rejects.
fn descriptor(value: &Value, at: &JsonPath, d: &mut Diagnostics) -> Option<BlockDescriptor> {
    // The JSON reader takes `tags` only as a string; any other `tags` is the shape message, not
    // a Molang reader error.
    let other_tags = value
        .as_object()
        .and_then(|o| o.get("tags"))
        .is_some_and(|tags| !tags.is_string());
    let descriptor = if other_tags {
        BlockDescriptor::Invalid(value.clone())
    } else {
        BlockDescriptor::from(value.clone())
    };
    for found in &descriptor.check(at) {
        d.push(found.clone());
    }
    (!matches!(descriptor, BlockDescriptor::Invalid(_))).then_some(descriptor)
}

fn above_block(value: &Value, at: &JsonPath, d: &mut Diagnostics) -> Option<AboveBlockFilter> {
    // Not an object: vanilla ignores it silently; the crate warns.
    let Some(o) = value.as_object() else {
        wrong_type(at, None, d);
        return None;
    };
    let blocks = o
        .get("blocks")
        .and_then(|v| blocks(v, &child(at, "blocks"), d));
    let kept: &[&str] = if blocks.is_none() && o.contains_key("blocks") {
        &["blocks"]
    } else {
        &[]
    };
    Some(AboveBlockFilter {
        blocks,
        distance: read::int(o.get("distance"), &child(at, "distance"), d),
        extra: members(o, &["blocks", "distance"], kept, at, d),
    })
}

/// An object or an array of objects; a non-object element is an `InvalidType` warning and
/// skipped (this crate's choice; vanilla adds a default entry), any
/// other value an `InvalidType` warning and the key ignored (vanilla: silently).
fn one_or_many<T>(
    value: &Value,
    at: &JsonPath,
    cx: &mut ReadCx<'_>,
    read_one: fn(&VObject, &JsonPath, &mut ReadCx<'_>) -> T,
) -> Option<OneOrMany<T>> {
    if let Some(array) = value.as_array() {
        let mut list = Vec::with_capacity(array.len());
        let mut places = Vec::with_capacity(array.len());
        for (i, entry) in array.iter().enumerate() {
            let at = element(at, i);
            match entry.as_object() {
                Some(o) => {
                    list.push(read_one(o, &at, cx));
                    places.push(i);
                }
                None => wrong_type(&at, None, cx.d),
            }
        }
        // What is told of an entry from here on is at the index the file has.
        cx.d.left_out(at, &places);
        return Some(OneOrMany::Many(list));
    }
    if let Some(o) = value.as_object() {
        return Some(OneOrMany::One(read_one(o, at, cx)));
    }
    wrong_type(at, None, cx.d);
    None
}

fn herd(o: &VObject, at: &JsonPath, cx: &mut ReadCx<'_>) -> Herd {
    let d = &mut *cx.d;
    Herd {
        min_size: read::int(o.get("min_size"), &child(at, "min_size"), d),
        max_size: read::int(o.get("max_size"), &child(at, "max_size"), d),
        event: read::string(o.get("event"), &child(at, "event"), d),
        event_skip_count: read::int(o.get("event_skip_count"), &child(at, "event_skip_count"), d),
        initial_event: read::string(o.get("initial_event"), &child(at, "initial_event"), d),
        initial_event_count: read::int(
            o.get("initial_event_count"),
            &child(at, "initial_event_count"),
            d,
        ),
        extra: unknown_members(
            o,
            &[
                "min_size",
                "max_size",
                "event",
                "event_skip_count",
                "initial_event",
                "initial_event_count",
            ],
            at,
            d,
        ),
    }
}

fn permutation(o: &VObject, at: &JsonPath, cx: &mut ReadCx<'_>) -> Permutation {
    let d = &mut *cx.d;
    let weight = read::int(o.get("weight"), &child(at, "weight"), d);
    // A weight below 1 is left out of the weighted pick without a word; the crate adds an
    // information.
    if weight.unwrap_or(1) < 1 {
        finding(invalid_value(None), Severity::Info, &child(at, "weight"), d);
    }
    Permutation {
        weight,
        guaranteed_count: read::int(o.get("guaranteed_count"), &child(at, "guaranteed_count"), d),
        entity_type: read::string(o.get("entity_type"), &child(at, "entity_type"), d)
            .map(EntityId::parse),
        extra: unknown_members(o, &["weight", "guaranteed_count", "entity_type"], at, d),
    }
}

fn brightness(value: &Value, at: &JsonPath, cx: &mut ReadCx<'_>) -> Option<BrightnessFilter> {
    let Some(o) = value.as_object() else {
        cx.object_form(texts::BRIGHTNESS_OBJECT, at);
        return None;
    };
    Some(BrightnessFilter {
        min: read::int(o.get("min"), &child(at, "min"), cx.d),
        max: read::int(o.get("max"), &child(at, "max"), cx.d),
        adjust_for_weather: read::boolean(
            o.get("adjust_for_weather"),
            &child(at, "adjust_for_weather"),
            cx.d,
        ),
        extra: unknown_members(o, &["min", "max", "adjust_for_weather"], at, cx.d),
    })
}

/// `min` / `max` of a range filter, with the swap warning.
fn int_range(
    o: &VObject,
    at: &JsonPath,
    cx: &mut ReadCx<'_>,
    swapped: &'static str,
) -> RangeFilter {
    let range = RangeFilter {
        min: read::int(o.get("min"), &child(at, "min"), cx.d),
        max: read::int(o.get("max"), &child(at, "max"), cx.d),
        extra: unknown_members(o, &["min", "max"], at, cx.d),
    };
    let (min, max) = range.ends();
    if min > max {
        cx.fixed(invalid_value, swapped, at);
    }
    range
}

fn height(value: &Value, at: &JsonPath, cx: &mut ReadCx<'_>) -> Option<RangeFilter> {
    let Some(o) = value.as_object() else {
        cx.object_form(texts::HEIGHT_OBJECT, at);
        return None;
    };
    Some(int_range(o, at, cx, texts::HEIGHT_SWAPPED))
}

fn distance(value: &Value, at: &JsonPath, cx: &mut ReadCx<'_>) -> Option<RangeFilter> {
    let Some(o) = value.as_object() else {
        cx.object_form(texts::DISTANCE_OBJECT, at);
        return None;
    };
    let range = int_range(o, at, cx, texts::DISTANCE_SWAPPED);
    // The spawn distances are clamped after the swap; the crate adds an information.
    let (min, max) = sorted(range.ends());
    if min < 0 || max > cx.profile.distance_cap {
        finding(
            DiagnosticCode::ClampedValue { text: None },
            Severity::Info,
            at,
            cx.d,
        );
    }
    Some(range)
}

fn world_age(value: &Value, at: &JsonPath, cx: &mut ReadCx<'_>) -> Option<WorldAgeFilter> {
    let Some(o) = value.as_object() else {
        cx.fixed(
            |text| DiagnosticCode::InvalidType { text },
            texts::WORLD_AGE_OBJECT,
            at,
        );
        return None;
    };
    let filter = WorldAgeFilter {
        min: read::int64(o.get("min"), &child(at, "min"), cx.d),
        max: read::int64(o.get("max"), &child(at, "max"), cx.d),
        extra: unknown_members(o, &["min", "max"], at, cx.d),
    };
    if filter.min.unwrap_or(0) > filter.max.unwrap_or(u64::MAX) {
        cx.fixed(invalid_value, texts::WORLD_AGE_SWAPPED, at);
    }
    Some(filter)
}

fn delay(value: &Value, at: &JsonPath, cx: &mut ReadCx<'_>) -> Option<DelayFilter> {
    let Some(o) = value.as_object() else {
        cx.object_form(texts::DELAY_OBJECT, at);
        return None;
    };
    let min = read::int(o.get("min"), &child(at, "min"), cx.d);
    let max = read::int(o.get("max"), &child(at, "max"), cx.d);
    if min.unwrap_or(0) > max.unwrap_or(0) {
        cx.formatted(invalid_value, texts::DELAY_SWAPPED, at);
    }
    let identifier = read::string(o.get("identifier"), &child(at, "identifier"), cx.d);
    if identifier.as_deref().is_none_or(str::is_empty) {
        cx.formatted(missing, texts::DELAY_WITHOUT_IDENTIFIER, at);
    }
    Some(DelayFilter {
        min,
        max,
        identifier: identifier.map(EntityId::parse),
        spawn_chance: read::int(o.get("spawn_chance"), &child(at, "spawn_chance"), cx.d),
        extra: unknown_members(o, &["min", "max", "identifier", "spawn_chance"], at, cx.d),
    })
}

fn mob_event(value: &Value, at: &JsonPath, cx: &mut ReadCx<'_>) -> Option<MobEventFilter> {
    let Some(o) = value.as_object() else {
        cx.object_form(texts::MOB_EVENT_OBJECT, at);
        return None;
    };
    Some(MobEventFilter {
        event: read::string(o.get("event"), &child(at, "event"), cx.d),
        extra: unknown_members(o, &["event"], at, cx.d),
    })
}

fn village(value: &Value, at: &JsonPath, cx: &mut ReadCx<'_>) -> Option<VillageFilter> {
    // Not an object: vanilla ignores it silently; the crate warns.
    let Some(o) = value.as_object() else {
        wrong_type(at, None, cx.d);
        return None;
    };
    if o.get("distance").is_none_or(Value::is_null) {
        cx.fixed(missing, texts::VILLAGE_DISTANCE_MISSING, at);
    }
    Some(VillageFilter {
        distance: read::uint(o.get("distance"), &child(at, "distance"), cx.d),
        village_border_tolerance: read::uint(
            o.get("village_border_tolerance"),
            &child(at, "village_border_tolerance"),
            cx.d,
        ),
        extra: unknown_members(o, &["distance", "village_border_tolerance"], at, cx.d),
    })
}

fn difficulty(value: &Value, at: &JsonPath, d: &mut Diagnostics) -> Option<DifficultyFilter> {
    // Not an object: peaceful..hard without a message in vanilla; the crate warns.
    let Some(o) = value.as_object() else {
        wrong_type(at, None, d);
        return None;
    };
    let mut end = |key: &str| {
        let at = child(at, key);
        let name = read::string(o.get(key), &at, d);
        // A name vanilla does not know takes the end's default silently: the crate adds an
        // information.
        let written = o.get(key).and_then(Value::as_string).is_some();
        if written
            && name
                .as_deref()
                .and_then(super::rules::Difficulty::from_name)
                .is_none()
        {
            finding(
                DiagnosticCode::InvalidEnumValue { text: None },
                Severity::Info,
                &at,
                d,
            );
        }
        name
    };
    let min = end("min");
    let max = end("max");
    Some(DifficultyFilter {
        min,
        max,
        extra: unknown_members(o, &["min", "max"], at, d),
    })
}

fn biome_filter(value: &Value, at: &JsonPath, d: &mut Diagnostics) -> Option<Filter> {
    if value.is_null() {
        return None;
    }
    match Filter::try_from(value.clone()) {
        Ok(filter) => {
            for found in &filter.check(at) {
                d.push(found.clone());
            }
            Some(filter)
        }
        Err(error) => {
            let path = at.join(&error.path);
            wrong_type(&path, None, d);
            None
        }
    }
}

/// `(min, max)` in ascending order.
pub fn sorted<T: PartialOrd>((min, max): (T, T)) -> (T, T) {
    if min > max { (max, min) } else { (min, max) }
}

impl RangeFilter {
    /// The ends with their defaults (24, 128), as written (not swapped).
    pub fn ends(&self) -> (i32, i32) {
        (
            self.min.unwrap_or(MIN_DEFAULT_SPAWN_DISTANCE),
            self.max.unwrap_or(MAX_DEFAULT_SPAWN_DISTANCE),
        )
    }
}

// ---- writing ----------------------------------------------------------------------------

/// A member of `object` when present.
fn put(object: &mut VObject, key: &str, value: Option<Value>) {
    if let Some(value) = value {
        object.insert(key, value);
    }
}

/// The unknown members after the known ones.
pub fn put_extra(object: &mut VObject, extra: &UnknownMembers) {
    for (key, value) in extra.iter() {
        object.insert(key, value.clone());
    }
}

fn int_value(value: Option<i32>) -> Option<Value> {
    value.map(|v| Value::from(i64::from(v)))
}

fn string_value(value: Option<&str>) -> Option<Value> {
    value.map(Value::from)
}

fn descriptors(list: &OneOrMany<BlockDescriptor>) -> Value {
    match list {
        OneOrMany::One(descriptor) => Value::from(descriptor),
        OneOrMany::Many(descriptors) => {
            Value::from(descriptors.iter().map(Value::from).collect::<VArray>())
        }
    }
}

fn entries<T>(list: &OneOrMany<T>, write: fn(&T) -> Value) -> Value {
    match list {
        OneOrMany::One(entry) => write(entry),
        OneOrMany::Many(entries) => Value::from(entries.iter().map(write).collect::<VArray>()),
    }
}

impl Condition {
    /// The condition as JSON, its members in vanilla's read order, unknown members last.
    #[allow(
        clippy::too_many_lines,
        reason = "one arm per key, in vanilla's read order"
    )]
    pub fn to_value(&self) -> Value {
        let mut o = VObject::new();
        let key = |i: usize| KEYS[i];
        put(
            &mut o,
            key(0),
            self.weight.as_ref().map(|w| {
                let mut m = VObject::new();
                put(&mut m, "default", int_value(w.default));
                put(&mut m, "rarity", int_value(w.rarity));
                put_extra(&mut m, &w.extra);
                Value::from(m)
            }),
        );
        put(
            &mut o,
            key(1),
            self.density_limit.as_ref().map(|l| {
                let mut m = VObject::new();
                put(&mut m, "surface", int_value(l.surface));
                put(&mut m, "underground", int_value(l.underground));
                put_extra(&mut m, &l.extra);
                Value::from(m)
            }),
        );
        put(
            &mut o,
            key(2),
            self.spawns_on_block_filter.as_ref().map(descriptors),
        );
        put(
            &mut o,
            key(3),
            self.spawns_on_block_prevented_filter
                .as_ref()
                .map(descriptors),
        );
        put(
            &mut o,
            key(4),
            self.spawns_above_block_filter.as_ref().map(|a| {
                let mut m = VObject::new();
                put(&mut m, "blocks", a.blocks.as_ref().map(descriptors));
                put(&mut m, "distance", int_value(a.distance));
                put_extra(&mut m, &a.extra);
                Value::from(m)
            }),
        );
        put(
            &mut o,
            key(5),
            self.herd.as_ref().map(|h| entries(h, herd_value)),
        );
        put(
            &mut o,
            key(6),
            self.permute_type
                .as_ref()
                .map(|p| entries(p, permutation_value)),
        );
        put(
            &mut o,
            key(7),
            self.brightness_filter.as_ref().map(|b| {
                let mut m = VObject::new();
                put(&mut m, "min", int_value(b.min));
                put(&mut m, "max", int_value(b.max));
                put(
                    &mut m,
                    "adjust_for_weather",
                    b.adjust_for_weather.map(Value::from),
                );
                put_extra(&mut m, &b.extra);
                Value::from(m)
            }),
        );
        put(&mut o, key(8), self.height_filter.as_ref().map(range_value));
        put(&mut o, key(9), self.spawns_on_surface.clone());
        put(&mut o, key(10), self.spawns_underground.clone());
        put(&mut o, key(11), self.spawns_underwater.clone());
        put(&mut o, key(12), self.spawns_lava.clone());
        put(&mut o, key(13), self.disallow_spawns_in_bubble.clone());
        put(
            &mut o,
            key(14),
            self.biome_filter
                .as_ref()
                .map(|f| Value::try_from(f).unwrap_or(Value::NULL)),
        );
        put(
            &mut o,
            key(15),
            self.difficulty_filter.as_ref().map(|f| {
                let mut m = VObject::new();
                put(&mut m, "min", string_value(f.min.as_deref()));
                put(&mut m, "max", string_value(f.max.as_deref()));
                put_extra(&mut m, &f.extra);
                Value::from(m)
            }),
        );
        put(
            &mut o,
            key(16),
            self.distance_filter.as_ref().map(range_value),
        );
        put(&mut o, key(17), self.is_experimental.clone());
        put(
            &mut o,
            key(18),
            self.world_age_filter.as_ref().map(|f| {
                let mut m = VObject::new();
                put(&mut m, "min", f.min.map(Value::from));
                put(&mut m, "max", f.max.map(Value::from));
                put_extra(&mut m, &f.extra);
                Value::from(m)
            }),
        );
        put(
            &mut o,
            key(19),
            self.delay_filter.as_ref().map(|f| {
                let mut m = VObject::new();
                put(&mut m, "min", int_value(f.min));
                put(&mut m, "max", int_value(f.max));
                put(
                    &mut m,
                    "identifier",
                    string_value(f.identifier.as_ref().map(EntityId::as_str)),
                );
                put(&mut m, "spawn_chance", int_value(f.spawn_chance));
                put_extra(&mut m, &f.extra);
                Value::from(m)
            }),
        );
        put(
            &mut o,
            key(20),
            self.mob_event_filter.as_ref().map(|f| {
                let mut m = VObject::new();
                put(&mut m, "event", string_value(f.event.as_deref()));
                put_extra(&mut m, &f.extra);
                Value::from(m)
            }),
        );
        put(&mut o, key(21), self.is_persistent.clone());
        put(
            &mut o,
            key(22),
            self.player_in_village_filter.as_ref().map(|f| {
                let mut m = VObject::new();
                put(&mut m, "distance", f.distance.map(Value::from));
                put(
                    &mut m,
                    "village_border_tolerance",
                    f.village_border_tolerance.map(Value::from),
                );
                put_extra(&mut m, &f.extra);
                Value::from(m)
            }),
        );
        put(&mut o, key(23), self.spawn_event.clone());
        put_extra(&mut o, &self.extra);
        Value::from(o)
    }
}

fn range_value(range: &RangeFilter) -> Value {
    let mut m = VObject::new();
    put(&mut m, "min", int_value(range.min));
    put(&mut m, "max", int_value(range.max));
    put_extra(&mut m, &range.extra);
    Value::from(m)
}

fn herd_value(h: &Herd) -> Value {
    let mut m = VObject::new();
    put(&mut m, "min_size", int_value(h.min_size));
    put(&mut m, "max_size", int_value(h.max_size));
    put(&mut m, "event", string_value(h.event.as_deref()));
    put(&mut m, "event_skip_count", int_value(h.event_skip_count));
    put(
        &mut m,
        "initial_event",
        string_value(h.initial_event.as_deref()),
    );
    put(
        &mut m,
        "initial_event_count",
        int_value(h.initial_event_count),
    );
    put_extra(&mut m, &h.extra);
    Value::from(m)
}

fn permutation_value(p: &Permutation) -> Value {
    let mut m = VObject::new();
    put(&mut m, "weight", int_value(p.weight));
    put(&mut m, "guaranteed_count", int_value(p.guaranteed_count));
    put(
        &mut m,
        "entity_type",
        string_value(p.entity_type.as_ref().map(EntityId::as_str)),
    );
    put_extra(&mut m, &p.extra);
    Value::from(m)
}
