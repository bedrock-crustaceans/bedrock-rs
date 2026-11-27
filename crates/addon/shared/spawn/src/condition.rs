//! The legacy spawn condition (`conditions[]` of 1.8.0-shaped spawn rules and of spawn groups):
//! its members as written, read by the derived reader under the game's jsoncpp accessors
//! (`Scalars::Accessors`, [`legacy::read`](super::legacy::read)), with a `read_with` fn for each
//! member the game reads by a rule of its own, and the writer.

use facet::Facet;
use facet_value::{VArray, VObject};

use super::codes;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, CodeDef, Diagnostic, Severity};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::EntityId;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::jsoncpp::accessors::{self, Integer};
use bedrock_addon_core::json::jsoncpp::lenient;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::schema::{self, ElementFailure, FromJson, Schema, read_list};
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang_with = super::legacy::visit_condition)]
pub struct Condition {
    /// The condition's spawn weight (JSON `minecraft:weight`).
    #[facet(
        rename = "minecraft:weight",
        schema::reads_null,
        schema::or_kept = "told"
    )]
    #[facet(schema::invalid = codes::WEIGHT_OBJECT)]
    #[facet(schema::read_with = object_form::<Weight>)]
    pub weight: Option<Weight>,
    /// How many of the entity's pool may exist (JSON `minecraft:density_limit`).
    #[facet(
        rename = "minecraft:density_limit",
        schema::reads_null,
        schema::or_kept = "told"
    )]
    #[facet(schema::invalid = codes::DENSITY_OBJECT)]
    #[facet(schema::read_with = object_form::<DensityLimit>)]
    pub density_limit: Option<DensityLimit>,
    /// The blocks the entity may spawn on (JSON `minecraft:spawns_on_block_filter`).
    #[facet(rename = "minecraft:spawns_on_block_filter", schema::reads_null)]
    #[facet(schema::or_kept = "told", schema::read_with = blocks)]
    pub spawns_on_block_filter: Option<OneOrMany<BlockDescriptor>>,
    /// The blocks the entity may not spawn on (JSON
    /// `minecraft:spawns_on_block_prevented_filter`).
    #[facet(
        rename = "minecraft:spawns_on_block_prevented_filter",
        schema::reads_null
    )]
    #[facet(schema::or_kept = "told", schema::read_with = blocks)]
    pub spawns_on_block_prevented_filter: Option<OneOrMany<BlockDescriptor>>,
    /// Blocks that must be found above the spawn position (JSON
    /// `minecraft:spawns_above_block_filter`).
    #[facet(rename = "minecraft:spawns_above_block_filter", schema::reads_null)]
    #[facet(schema::or_kept = "told")]
    pub spawns_above_block_filter: Option<AboveBlockFilter>,
    /// How many spawn at once, and the events they get (JSON `minecraft:herd`, one object or an
    /// array).
    #[facet(
        rename = "minecraft:herd",
        schema::reads_null,
        schema::or_kept = "told"
    )]
    pub herd: Option<OneOrMany<Herd>>,
    /// Other entities spawned in the entity's place (JSON `minecraft:permute_type`, one object
    /// or an array).
    #[facet(
        rename = "minecraft:permute_type",
        schema::reads_null,
        schema::or_kept = "told"
    )]
    pub permute_type: Option<OneOrMany<Permutation>>,
    /// The light level range (JSON `minecraft:brightness_filter`).
    #[facet(rename = "minecraft:brightness_filter", schema::reads_null)]
    #[facet(schema::or_kept = "told", schema::invalid = codes::BRIGHTNESS_OBJECT)]
    #[facet(schema::read_with = object_form::<BrightnessFilter>)]
    pub brightness_filter: Option<BrightnessFilter>,
    /// The height range (JSON `minecraft:height_filter`).
    #[facet(
        rename = "minecraft:height_filter",
        schema::reads_null,
        schema::or_kept = "told"
    )]
    #[facet(schema::invalid = codes::HEIGHT_OBJECT, schema::read_with = height)]
    pub height_filter: Option<RangeFilter>,
    /// Spawns on the surface; vanilla tests only that the member exists, whatever its value
    /// (JSON `minecraft:spawns_on_surface`, the value as written).
    #[facet(rename = "minecraft:spawns_on_surface", schema::reads_null)]
    pub spawns_on_surface: Option<Value>,
    /// Spawns underground; present = set (JSON `minecraft:spawns_underground`).
    #[facet(rename = "minecraft:spawns_underground", schema::reads_null)]
    pub spawns_underground: Option<Value>,
    /// Spawns in water; present = set (JSON `minecraft:spawns_underwater`).
    #[facet(rename = "minecraft:spawns_underwater", schema::reads_null)]
    pub spawns_underwater: Option<Value>,
    /// Spawns in lava; present = set (JSON `minecraft:spawns_lava`).
    #[facet(rename = "minecraft:spawns_lava", schema::reads_null)]
    pub spawns_lava: Option<Value>,
    /// Never spawns in a bubble column; present = set (JSON
    /// `minecraft:disallow_spawns_in_bubble`).
    #[facet(rename = "minecraft:disallow_spawns_in_bubble", schema::reads_null)]
    pub disallow_spawns_in_bubble: Option<Value>,
    /// The biomes the condition applies in; absent = every biome (JSON `minecraft:biome_filter`).
    #[facet(
        rename = "minecraft:biome_filter",
        schema::reads_null,
        schema::or_kept = "told"
    )]
    #[facet(schema::read_with = biome_filter)]
    pub biome_filter: Option<Filter>,
    /// The world difficulty range (JSON `minecraft:difficulty_filter`).
    #[facet(rename = "minecraft:difficulty_filter", schema::reads_null)]
    #[facet(schema::or_kept = "told")]
    pub difficulty_filter: Option<DifficultyFilter>,
    /// The distance range from the nearest player (JSON `minecraft:distance_filter`).
    #[facet(rename = "minecraft:distance_filter", schema::reads_null)]
    #[facet(schema::or_kept = "told", schema::invalid = codes::DISTANCE_OBJECT)]
    #[facet(schema::read_with = distance)]
    pub distance_filter: Option<RangeFilter>,
    /// Only with the experimental gameplay toggle; present = set (JSON
    /// `minecraft:is_experimental`).
    #[facet(rename = "minecraft:is_experimental", schema::reads_null)]
    pub is_experimental: Option<Value>,
    /// The world age range in ticks (JSON `minecraft:world_age_filter`).
    #[facet(rename = "minecraft:world_age_filter", schema::reads_null)]
    #[facet(schema::or_kept = "told", schema::invalid = codes::WORLD_AGE_OBJECT)]
    #[facet(schema::read_with = object_form::<WorldAgeFilter>)]
    pub world_age_filter: Option<WorldAgeFilter>,
    /// A delay before the spawn (JSON `minecraft:delay_filter`).
    #[facet(
        rename = "minecraft:delay_filter",
        schema::reads_null,
        schema::or_kept = "told"
    )]
    #[facet(schema::invalid = codes::DELAY_OBJECT)]
    #[facet(schema::read_with = object_form::<DelayFilter>)]
    pub delay_filter: Option<DelayFilter>,
    /// A mob event that must be on (JSON `minecraft:mob_event_filter`).
    #[facet(rename = "minecraft:mob_event_filter", schema::reads_null)]
    #[facet(schema::or_kept = "told", schema::invalid = codes::MOB_EVENT_OBJECT)]
    #[facet(schema::read_with = object_form::<MobEventFilter>)]
    pub mob_event_filter: Option<MobEventFilter>,
    /// Spawned entities never despawn; present = set (JSON `minecraft:is_persistent`).
    #[facet(rename = "minecraft:is_persistent", schema::reads_null)]
    pub is_persistent: Option<Value>,
    /// Only near a village a player is in (JSON `minecraft:player_in_village_filter`).
    #[facet(rename = "minecraft:player_in_village_filter", schema::reads_null)]
    #[facet(schema::or_kept = "told")]
    pub player_in_village_filter: Option<VillageFilter>,
    /// Kept as written; vanilla does not read it (JSON `minecraft:spawn_event`).
    #[facet(rename = "minecraft:spawn_event", schema::reads_null)]
    pub spawn_event: Option<Value>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// `minecraft:weight`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Weight {
    /// The condition's weight; absent = 100, a negative value counts as 0 (JSON `default`).
    pub default: Option<i32>,
    /// The rarity; absent = 0, `null` = 5 (JSON `rarity`).
    #[facet(schema::reads_null, schema::read_with = rarity)]
    pub rarity: Option<i32>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// `minecraft:density_limit`: population caps, −1 for none.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct DensityLimit {
    /// The cap on the surface; absent = −1 (JSON `surface`).
    pub surface: Option<i32>,
    /// The cap underground; absent = −1 (JSON `underground`).
    pub underground: Option<i32>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// `minecraft:spawns_above_block_filter`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct AboveBlockFilter {
    /// The blocks (JSON `blocks`).
    #[facet(schema::reads_null, schema::or_kept = "told", schema::read_with = blocks)]
    pub blocks: Option<OneOrMany<BlockDescriptor>>,
    /// How far above; absent = 1 (JSON `distance`).
    pub distance: Option<i32>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// One entry of `minecraft:herd` (one herd). No `min_size ≤ max_size` check;
/// a negative size wraps to a large count, as in vanilla.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Permutation {
    /// The weight in the weighted pick; absent = 1, below 1 leaves the entity out of it (JSON
    /// `weight`).
    #[facet(schema::read_with = permutation_weight)]
    pub weight: Option<i32>,
    /// How many are spawned regardless of the pick; absent = 0 (JSON `guaranteed_count`).
    pub guaranteed_count: Option<i32>,
    /// The entity (JSON `entity_type`).
    #[facet(schema::read_with = entity)]
    pub entity_type: Option<EntityId>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// `minecraft:brightness_filter`: the light level range, each end clamped to 0..=15.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct DifficultyFilter {
    /// The lowest difficulty; absent or unknown = `peaceful` (JSON `min`).
    #[facet(schema::read_with = difficulty)]
    pub min: Option<String>,
    /// The highest difficulty; absent or unknown = `hard` (JSON `max`).
    #[facet(schema::read_with = difficulty)]
    pub max: Option<String>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// `minecraft:world_age_filter`: a range of world ages in ticks, compared unsigned (a negative
/// number is stored as its two's complement).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::after_read = world_age_swapped)]
pub struct WorldAgeFilter {
    /// The lowest age; absent = 0 (JSON `min`).
    pub min: Option<u64>,
    /// The highest age; absent = `u64::MAX` (JSON `max`).
    pub max: Option<u64>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// `minecraft:delay_filter`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct DelayFilter {
    /// The shortest delay; absent = 0, a negative one counts as 0 (JSON `min`).
    pub min: Option<i32>,
    /// The longest delay; absent = 0 (JSON `max`).
    pub max: Option<i32>,
    /// The entity the delay is for (JSON `identifier`).
    #[facet(schema::guard = delay_swapped, schema::read_with = entity)]
    pub identifier: Option<EntityId>,
    /// The chance in percent; absent = 100 (JSON `spawn_chance`).
    #[facet(schema::guard = delay_identified)]
    pub spawn_chance: Option<i32>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// `minecraft:mob_event_filter`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct MobEventFilter {
    /// The mob event's name (JSON `event`).
    pub event: Option<String>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// `minecraft:player_in_village_filter`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::after_members = village_distance)]
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

// ---- the members the game reads by a rule of its own ---------------------------------------

/// A condition text at `node`: the legacy parsers log each at code 2, level 2 in both masks.
fn warn(read: &mut Read<'_>, node: &Node<'_>, code: Code) {
    let mut finding = Diagnostic::of(code)
        .severity(Severity::Warning)
        .at(node.path().clone());
    finding.location.span = node.span();
    read.push(finding);
}

/// The crate's own information `code` at `node`: vanilla is silent.
fn inform(read: &mut Read<'_>, node: &Node<'_>, code: &'static CodeDef<0>) {
    let mut finding = Diagnostic::of(code)
        .severity(Severity::Info)
        .at(node.path().clone());
    finding.location.span = node.span();
    read.push(finding);
}

/// A member vanilla reads only from an object: another value (`null` too) is the field's
/// object-form warning (`schema::invalid`, fed the identifier) and the member is kept as written;
/// an object is read as `T`.
fn object_form<T: FromJson>(node: &Node<'_>, read: &mut Read<'_>) -> Option<T> {
    if !node.value().is_object() {
        if let Some(code) = read.invalid_code(node) {
            warn(read, node, code);
        }
        return None;
    }
    T::from_json(node, read)
}

/// `code` fed the document's identifier (the read's subject).
fn named(read: &Read<'_>, code: &'static CodeDef<1>) -> Code {
    code.with([read.subject_text().unwrap_or_default()])
}

/// `rarity`: read only when the member exists, with a default of 5: a `null` is 5, not the 0 of
/// an absent member. The accessor tells a `null` at the member, since it is written as the 5 it
/// is read as.
fn rarity(node: &Node<'_>, read: &mut Read<'_>) -> Option<i32> {
    let rarity = i32::from_json(node, read)?;
    Some(if node.value().is_null() { 5 } else { rarity })
}

/// A block list: an empty value (`null`, `[]`, `{}`) adds nothing, an array one descriptor per
/// element, anything else one descriptor. A descriptor that fails gets its `InvalidValue`
/// warning and is dropped (this crate's choice; vanilla keeps an empty descriptor). `None` (the
/// member kept as written) for `null`, `{}` and a single descriptor that fails: `null` and `{}`
/// with no finding, as vanilla reads them.
fn blocks(node: &Node<'_>, read: &mut Read<'_>) -> Option<OneOrMany<BlockDescriptor>> {
    let value = node.value();
    if value.is_null() || value.as_object().is_some_and(VObject::is_empty) {
        return None;
    }
    if value.is_array() {
        return read_list(node, read, ElementFailure::Drop, descriptor).map(OneOrMany::Many);
    }
    descriptor(node, read).map(OneOrMany::One)
}

/// One descriptor and its findings; `None` for a shape vanilla's reader rejects.
fn descriptor(node: &Node<'_>, read: &mut Read<'_>) -> Option<BlockDescriptor> {
    let value = node.value();
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
    // The spawn families read under the first-run profile, whose mask tells an unknown
    // member as a warning.
    for found in &descriptor.check(node.path(), Severity::Warning) {
        read.push(found.clone());
    }
    (!matches!(descriptor, BlockDescriptor::Invalid(_))).then_some(descriptor)
}

/// An entity name, read as `asString` reads it.
fn entity(node: &Node<'_>, read: &mut Read<'_>) -> Option<EntityId> {
    String::from_json(node, read).map(EntityId::parse)
}

/// A permutation's `weight`: below 1 it leaves the entity out of the weighted pick without a
/// word; the crate adds an information.
fn permutation_weight(node: &Node<'_>, read: &mut Read<'_>) -> Option<i32> {
    let weight = i32::from_json(node, read)?;
    if weight < 1 {
        inform(read, node, &core_codes::INVALID_VALUE);
    }
    Some(weight)
}

/// A range filter, and the swap warning `swapped` after its members.
fn int_range(
    node: &Node<'_>,
    read: &mut Read<'_>,
    swapped: &'static CodeDef<0>,
) -> Option<RangeFilter> {
    let range = object_form::<RangeFilter>(node, read)?;
    let (min, max) = range.ends();
    if min > max {
        warn(read, node, Code::from(swapped));
    }
    Some(range)
}

fn height(node: &Node<'_>, read: &mut Read<'_>) -> Option<RangeFilter> {
    int_range(node, read, &codes::HEIGHT_SWAPPED)
}

/// The parser's profile, by the payload key the member lies under.
fn profile_at(node: &Node<'_>) -> Profile {
    let groups = node.path().segments().next()
        == Some(bedrock_addon_core::diagnostic::JsonPathSegment::Key(
            super::legacy::GROUPS_KEY,
        ));
    if groups { SPAWN_GROUPS } else { SPAWN_RULES }
}

fn distance(node: &Node<'_>, read: &mut Read<'_>) -> Option<RangeFilter> {
    let range = int_range(node, read, &codes::DISTANCE_SWAPPED)?;
    // The spawn distances are clamped after the swap; the crate adds an information.
    let (min, max) = sorted(range.ends());
    if min < 0 || max > profile_at(node).distance_cap {
        inform(read, node, &core_codes::CLAMPED_VALUE);
    }
    Some(range)
}

/// A difficulty name: one vanilla does not know takes the end's default silently; the crate adds
/// an information when the name is written as a string.
fn difficulty(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    let name = String::from_json(node, read)?;
    if node.value().is_string() && super::rules::Difficulty::from_name(&name).is_none() {
        inform(read, node, &core_codes::INVALID_ENUM_VALUE);
    }
    Some(name)
}

/// The biome filter as the filter reader takes it: `null` adds nothing (the member kept as
/// written, no finding); a value no filter reads is an `InvalidType` information where it fails.
fn biome_filter(node: &Node<'_>, read: &mut Read<'_>) -> Option<Filter> {
    let value = node.value();
    if value.is_null() {
        return None;
    }
    match Filter::try_from(value.clone()) {
        Ok(filter) => {
            // An unknown member is a warning, as at `descriptor`.
            for found in &filter.check(node.path(), Severity::Warning) {
                read.push(found.clone());
            }
            Some(filter)
        }
        Err(error) => {
            read.push(
                Diagnostic::of(&core_codes::INVALID_TYPE)
                    .severity(Severity::Info)
                    .at(node.path().join(&error.path)),
            );
            None
        }
    }
}

/// `world_age_filter.min > max`, after its members and its unknown members.
fn world_age_swapped(filter: &mut WorldAgeFilter, node: &Node<'_>, read: &mut Read<'_>) {
    if filter.min.unwrap_or(0) > filter.max.unwrap_or(u64::MAX) {
        warn(read, node, Code::from(&codes::WORLD_AGE_SWAPPED));
    }
}

/// A delay bound as the game binds it (`asInt`; absent or `null` 0).
fn delay_bound(filter: &Node<'_>, key: &str) -> i128 {
    match filter
        .value()
        .as_object()
        .and_then(|members| members.get(key))
    {
        Some(value) => match accessors::as_integer(value, 32, true) {
            Integer::Fits(bound) | Integer::Beyond(bound) | Integer::Coerced(bound) => bound,
        },
        None => 0,
    }
}

/// `schema::guard` of `delay_filter.identifier`: `min > max`, told where the game's reader
/// compares them, after binding `max`.
fn delay_swapped(filter: &Node<'_>, read: &mut Read<'_>) -> bool {
    if delay_bound(filter, "min") > delay_bound(filter, "max") {
        warn(read, filter, named(read, &codes::DELAY_SWAPPED));
    }
    true
}

/// `schema::guard` of `delay_filter.spawn_chance`: a missing, `null` or empty `identifier`,
/// told after binding it.
fn delay_identified(filter: &Node<'_>, read: &mut Read<'_>) -> bool {
    let identifier = filter
        .value()
        .as_object()
        .and_then(|members| members.get("identifier"))
        .map(|value| EntityId::parse(lenient::to_text(Some(value))));
    if identifier.is_none_or(|identifier| identifier.as_str().is_empty()) {
        warn(read, filter, named(read, &codes::DELAY_WITHOUT_IDENTIFIER));
    }
    true
}

/// `player_in_village_filter.distance` missing or `null`, after its members and before its
/// unknown members.
fn village_distance(filter: &mut VillageFilter, node: &Node<'_>, read: &mut Read<'_>) {
    if filter.distance.is_none() {
        warn(read, node, Code::from(&codes::VILLAGE_DISTANCE_MISSING));
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
