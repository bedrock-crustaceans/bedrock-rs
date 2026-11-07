//! The entity family's codes: the game's content-log lines for behaviour-pack entity files, byte
//! for byte, each a definition beside the rules that raise it. Each [`was`](CodeDef::was) the legacy
//! code its sites raised, so a pin or a filter on the code's name keeps matching.
//!
//! A line the game logs at the error level after which the entity loads is a kept error
//! ([`Class::KeptError`]). The crate's own sentences that the legacy code carried as a text (the
//! upgrade's removals, the handler lookup) keep it as their [`game`](CodeDef::game) sentence, so
//! the finding reads as it did.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Disposition, Kind};

// ---- the document ----

/// A document above the engine, told in the resource pack's words: the version as written and the
/// engine's.
pub static ACTOR_ABOVE_ENGINE: CodeDef<2> = CodeDef::new(
    "entity/actor_above_engine",
    Kind::UnsupportedVersion,
    Class::Error,
    "format version {} is above the engine's {}",
)
.game("Error: actor resource file has format version: {} exceeds game version: {}. Skipping.")
.was("UnsupportedVersion");

/// The line the game writes when the load of an entity carries errors: the identifier, and the
/// first error with its log's prefix (`parse errors occurred: `, `upgrade errors occurred: `,
/// `validate errors occurred: `).
pub static FAILED_TO_LOAD: CodeDef<2> = CodeDef::new(
    "entity/failed_to_load",
    Kind::Closing,
    Class::Error,
    "the entity {} failed to load: {}",
)
.game("ERROR: Entity '{}' failed to load from JSON: {}")
.was("DocumentRejected");

// ---- the description ----

/// `aliases` in a document at or above 1.21.10: the game logs it at the error level and loads the
/// entity, so the crate reports a warning.
pub static ALIASES_DEPRECATED: CodeDef<0> = CodeDef::new(
    "entity/aliases_deprecated",
    Kind::Deprecated,
    Class::Warning,
    "aliases are deprecated",
)
.game("Deprecated field: Aliases.")
.was("DeprecatedMember");

/// One bad property drops the whole property group; the entity loads.
pub static ACTOR_PROPERTIES_NOT_LOADED: CodeDef<0> = CodeDef::new(
    "entity/actor_properties_not_loaded",
    Kind::UnknownMember(Disposition::Dropped),
    Class::KeptError,
    "the properties are dropped",
)
.game("Error loading Actor Properties")
.was("DroppedField");

/// An entry of `scripts.animate` that is an object of other than one member, of its member count.
pub static ANIMATION_SCRIPT_OBJECT_SIZE: CodeDef<1> = CodeDef::new(
    "entity/animation_script_object_size",
    Kind::InvalidValue,
    Class::KeptError,
    "an animation script object of {} members",
)
.game("Error: animation script object has {} items - size must be 1.  Skipping.")
.was("InvalidValue");

// ---- entity_version ----

/// An `entity_version` that is not an object: the entity loads versionless.
pub static VERSION_NOT_OBJECT: CodeDef<0> = CodeDef::new(
    "entity/version_not_object",
    Kind::InvalidValue,
    Class::KeptError,
    "entity_version is not an object",
)
.game("Error loading 'entity_version' as it is not a JSON object.")
.was("InvalidValue");

/// An `entity_version` whose `upgrades` is not an array.
pub static UPGRADES_NOT_ARRAY: CodeDef<0> = CodeDef::new(
    "entity/upgrades_not_array",
    Kind::InvalidValue,
    Class::KeptError,
    "entity_version.upgrades is not an array",
)
.game("Error loading 'entity_version' as 'upgrades' is not an array.")
.was("InvalidValue");

/// An entry of `upgrades` that is not an object.
pub static UPGRADE_ENTRY_NOT_OBJECT: CodeDef<0> = CodeDef::new(
    "entity/upgrade_entry_not_object",
    Kind::InvalidValue,
    Class::KeptError,
    "an upgrade entry is not an object",
)
.game("Error loading 'entity_version' as 'upgrades' contains an entry that is not a JSON object.")
.was("InvalidValue");

/// An upgrade entry without an integer `upgrade_to`.
pub static NO_INTEGER_UPGRADE_TO: CodeDef<0> = CodeDef::new(
    "entity/no_integer_upgrade_to",
    Kind::InvalidValue,
    Class::KeptError,
    "an upgrade entry without an integer upgrade_to",
)
.game(
    "Error loading 'entity_version' as an upgrade entry does not contain an integer 'upgrade_to'.",
)
.was("InvalidValue");

/// An upgrade entry without a string `name`.
pub static NO_STRING_NAME: CodeDef<0> = CodeDef::new(
    "entity/no_string_name",
    Kind::InvalidValue,
    Class::KeptError,
    "an upgrade entry without a string name",
)
.game("Error loading 'entity_version' as an upgrade entry does not contain a string 'name'.")
.was("InvalidValue");

/// An upgrade entry whose `name` is empty.
pub static EMPTY_NAME: CodeDef<0> = CodeDef::new(
    "entity/empty_name",
    Kind::InvalidValue,
    Class::KeptError,
    "an upgrade entry with an empty name",
)
.game("Error loading 'entity_version' as an upgrade entry's 'name' is empty.")
.was("InvalidValue");

/// An upgrade entry whose `upgrade_to` is 0, the versionless entities' version.
pub static UPGRADE_TO_ZERO: CodeDef<0> = CodeDef::new(
    "entity/upgrade_to_zero",
    Kind::InvalidValue,
    Class::KeptError,
    "an upgrade entry to version 0",
)
.game("Error loading 'entity_version' as an upgrade entry 'upgrade_to' version is 0, which is reserved for versionless entities.")
.was("InvalidValue");

/// Upgrade versions whose lowest is not 1.
pub static VERSION_NOT_FROM_ONE: CodeDef<0> = CodeDef::new(
    "entity/version_not_from_one",
    Kind::InvalidValue,
    Class::KeptError,
    "the upgrade versions do not start at 1",
)
.game("Error loading 'entity_version' as its version does not start at 1.")
.was("InvalidValue");

/// Two upgrade entries for one version, of the version.
pub static VERSION_DUPLICATE: CodeDef<1> = CodeDef::new(
    "entity/version_duplicate",
    Kind::InvalidValue,
    Class::KeptError,
    "two upgrade entries for version {}",
)
.game("Error loading 'entity_version' as it contains multiple upgrade entries for version {}.")
.was("InvalidValue");

/// Upgrade versions with a gap: the entity's identifier and the versions either side of it.
pub static VERSION_GAP: CodeDef<3> = CodeDef::new(
    "entity/version_gap",
    Kind::InvalidValue,
    Class::KeptError,
    "{} skips the versions between {} and {}",
)
.game("Error loading '{}' as it contains a non-contiguous version between {} and {}.")
.was("InvalidValue");

/// An upgrade entry naming a handler `upgrades` does not define. The crate's own sentence: the
/// game's failed lookup has no text.
pub static UPGRADE_HANDLER_UNDEFINED: CodeDef<1> = CodeDef::new(
    "entity/upgrade_handler_undefined",
    Kind::InvalidValue,
    Class::Lint,
    "no upgrade handler {}",
)
.game("upgrade handler '{}' is not defined in 'upgrades'")
.was("InvalidValue");

// ---- the upgrade ----

/// A component the game's upgrade drops in silence: the component and the step's version. The
/// crate's own sentence.
pub static COMPONENT_REMOVED_BY_UPGRADE: CodeDef<2> = CodeDef::new(
    "entity/component_removed_by_upgrade",
    Kind::UnknownMember(Disposition::Dropped),
    Class::Lint,
    "{} removed by the {} upgrade",
)
.game("`{}` was removed by the {} upgrade")
.was("DroppedComponent");

/// A member the game's upgrade drops in silence: the member and the step's version. The crate's
/// own sentence.
pub static MEMBER_REMOVED_BY_UPGRADE: CodeDef<2> = CodeDef::new(
    "entity/member_removed_by_upgrade",
    Kind::UnknownMember(Disposition::Dropped),
    Class::Lint,
    "{} removed by the {} upgrade",
)
.game("`{}` was removed by the {} upgrade")
.was("DroppedField");

/// A component group whose value is `null`, removed by the upgrade. The crate's own sentence.
pub static NULL_GROUP_REMOVED: CodeDef<1> = CodeDef::new(
    "entity/null_group_removed",
    Kind::UnknownMember(Disposition::Dropped),
    Class::Lint,
    "the null component group {} is removed",
)
.game("component group '{}' is null and was removed")
.was("DroppedComponent");

/// `minecraft:damage_sensor` in a document below 1.15, which the 1.13.0 step removes.
pub static DAMAGE_SENSOR_NOT_ADDED: CodeDef<0> = CodeDef::new(
    "entity/damage_sensor_not_added",
    Kind::UnknownMember(Disposition::Dropped),
    Class::Lint,
    "minecraft:damage_sensor needs format version 1.15",
)
.game("The component minecraft:damage_sensor will not be added to this entity as it won't work unless the format version is 1.15 or greater.")
.was("DroppedComponent");

// ---- components ----

/// A string the member's table does not know: the value as written and the member.
pub static NOT_A_VALID_VALUE: CodeDef<2> = CodeDef::new(
    "entity/not_a_valid_value",
    Kind::InvalidValue,
    Class::KeptError,
    "{} is no valid {}",
)
.game("'{}' is not a valid '{}' value.")
.was("InvalidValue");

/// The legacy `transformation` reader's line for a `delay` of another type; the delay keeps its
/// default.
pub static DELAY_NOT_NUMBER_OR_OBJECT: CodeDef<0> = CodeDef::new(
    "entity/delay_not_number_or_object",
    Kind::InvalidValue,
    Class::KeptError,
    "a delay that is neither a number nor an object",
)
.game("Expected Number or Object.")
.was("InvalidValue");

/// A `days_until_insomnia` that is no number, at the typed version.
pub static INSOMNIA_NOT_A_NUMBER: CodeDef<0> = CodeDef::new(
    "entity/insomnia_not_a_number",
    Kind::WrongType,
    Class::Error,
    "days_until_insomnia is not a number",
)
.game("days_until_insomnia: invalid numeric value")
.was("InvalidType");

/// A string an enum node of the game's component schema does not know: the value and the keys, as
/// the game lists them.
pub static ENUM_NOT_ALLOWED: CodeDef<2> = CodeDef::new(
    "entity/enum_not_allowed",
    Kind::InvalidEnumValue,
    Class::KeptError,
    "value {} is none of the keys {}",
)
.game("Value '{}' not allowed.  Must match at least one of the following keys: {}")
.was("InvalidEnumValue");

// ---- validation ----

/// `minecraft:spawn_egg_interact` in a section without `minecraft:offspring`: the definition stays.
pub static SPAWN_EGG_NEEDS_OFFSPRING: CodeDef<0> = CodeDef::new(
    "entity/spawn_egg_needs_offspring",
    Kind::MissingRequired,
    Class::KeptError,
    "minecraft:spawn_egg_interact without minecraft:offspring",
)
.game("\"minecraft:spawn_egg_interact\" requires \"minecraft:offspring\" to be present on the same entity definition.")
.was("MissingRequiredField");

/// A timer with both `time` and `random_time_choices`: the game logs it at the warning level.
pub static TIMER_BOTH: CodeDef<0> = CodeDef::new(
    "entity/timer_both",
    Kind::UnknownMember(Disposition::Ignored),
    Class::Warning,
    "a timer with both time and random_time_choices",
)
.game("Both \"time\" and \"random_time_choices\" have been specified for a timer. Defaulting to \"time\".")
.was("IgnoredMember");

/// A preferred path with a block in two costing groups, of the block's name.
pub static DUPLICATE_PATH_BLOCK: CodeDef<1> = CodeDef::new(
    "entity/duplicate_path_block",
    Kind::InvalidValue,
    Class::Error,
    "the block {} is in two costing groups",
)
.game("Duplicate block descriptors in two costing groups {}")
.was("InvalidValue");

/// A member of an older filter group that is neither a group key nor a member the game maps to a
/// test.
pub static INVALID_FILTER_MEMBER: CodeDef<1> = CodeDef::new(
    "entity/invalid_filter_member",
    Kind::UnknownMember(Disposition::Kept),
    Class::KeptError,
    "{} is no filter group member",
)
.game("Invalid EntityFilterGroup member found: {}")
.was("UnrecognizedField");

// ---- goals ----

/// A `search_distance` with a value that is not positive: the default is used.
pub static SEARCH_DISTANCE: CodeDef<0> = CodeDef::new(
    "entity/search_distance",
    Kind::InvalidValue,
    Class::KeptError,
    "search_distance holds a value that is not positive",
)
.game("behavior.transport_items: \"search_distance\" may only contain positive values")
.was("InvalidValue");

/// A block of `eat_and_replace_block_pairs` that is not one, of the name as written: the pair is
/// dropped.
pub static EAT_BLOCK_INVALID_BLOCK: CodeDef<1> = CodeDef::new(
    "entity/eat_block_invalid_block",
    Kind::UnresolvedReference,
    Class::KeptError,
    "{} is no block",
)
.game("Block {} on \"eat_and_replace_block_pairs\" on \"minecraft:behavior.eat_block\" is not a valid block.")
.was("UnresolvedReference");

/// Every definition of the family, for the registry of ids.
pub static ALL: &[&CodeMeta] = &[
    ACTOR_ABOVE_ENGINE.meta(),
    FAILED_TO_LOAD.meta(),
    ALIASES_DEPRECATED.meta(),
    ACTOR_PROPERTIES_NOT_LOADED.meta(),
    ANIMATION_SCRIPT_OBJECT_SIZE.meta(),
    VERSION_NOT_OBJECT.meta(),
    UPGRADES_NOT_ARRAY.meta(),
    UPGRADE_ENTRY_NOT_OBJECT.meta(),
    NO_INTEGER_UPGRADE_TO.meta(),
    NO_STRING_NAME.meta(),
    EMPTY_NAME.meta(),
    UPGRADE_TO_ZERO.meta(),
    VERSION_NOT_FROM_ONE.meta(),
    VERSION_DUPLICATE.meta(),
    VERSION_GAP.meta(),
    UPGRADE_HANDLER_UNDEFINED.meta(),
    COMPONENT_REMOVED_BY_UPGRADE.meta(),
    MEMBER_REMOVED_BY_UPGRADE.meta(),
    NULL_GROUP_REMOVED.meta(),
    DAMAGE_SENSOR_NOT_ADDED.meta(),
    NOT_A_VALID_VALUE.meta(),
    DELAY_NOT_NUMBER_OR_OBJECT.meta(),
    INSOMNIA_NOT_A_NUMBER.meta(),
    ENUM_NOT_ALLOWED.meta(),
    SPAWN_EGG_NEEDS_OFFSPRING.meta(),
    TIMER_BOTH.meta(),
    DUPLICATE_PATH_BLOCK.meta(),
    INVALID_FILTER_MEMBER.meta(),
    SEARCH_DISTANCE.meta(),
    EAT_BLOCK_INVALID_BLOCK.meta(),
];
