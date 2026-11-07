//! The item family's codes: the game's content-log lines for item files and the trim registries,
//! byte for byte, each a definition beside the reader, upgrade or rule that raises it. Each id is
//! `item/<name>`, the name its constant's in lower case. A sentence marked "the crate's own" is
//! worded by this crate (the game says nothing there, or its line is not known) and kept as the
//! sentence the legacy text carried.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Disposition, Kind};

// ---- the read and the legacy format ----

/// `minecraft:foil` at or above 1.20.20; the item loads on: a kept error.
pub static FOIL_DEPRECATED: CodeDef<0> = CodeDef::new(
    "item/foil_deprecated",
    Kind::Deprecated,
    Class::KeptError,
    "minecraft:foil is deprecated from 1.20.20",
)
.game("minecraft:foil has been deprecated in format versions 1.20.20 and above. Use minecraft:glint instead.");

/// `minecraft:foil` below 1.20.20 in a pack without the legacy creator features (the crate's own
/// sentence; the game says nothing): an information.
pub static FOIL_IGNORED: CodeDef<0> = CodeDef::new(
    "item/foil_ignored",
    Kind::UnknownMember(Disposition::Dropped),
    Class::Lint,
    "minecraft:foil is ignored for this pack",
)
.game("minecraft:foil is ignored for this pack; use minecraft:glint");

/// `minecraft:mining_speed` at or above 1.20.30: a kept error.
pub static MINING_SPEED_DEPRECATED: CodeDef<0> = CodeDef::new(
    "item/mining_speed_deprecated",
    Kind::Deprecated,
    Class::KeptError,
    "minecraft:mining_speed is deprecated from 1.20.30",
)
.game("minecraft:mining_speed has been deprecated in format versions 1.20.30 and above. Use minecraft:digger instead.");

/// A `tag:` key at or above 1.20.50: a kept error.
pub static TAG_DEPRECATED: CodeDef<0> = CodeDef::new(
    "item/tag_deprecated",
    Kind::Deprecated,
    Class::KeptError,
    "tag: is deprecated from 1.20.50",
)
.game("tag: has been deprecated in format versions 1.20.50 and above. Use the minecraft:tags component to store all tags instead.");

/// `minecraft:creative_category` at or above 1.20.20; the argument is the document's version:
/// a kept error.
pub static CREATIVE_CATEGORY_DEPRECATED: CodeDef<1> = CodeDef::new(
    "item/creative_category_deprecated",
    Kind::Deprecated,
    Class::KeptError,
    "minecraft:creative_category is deprecated at format_version {}",
)
.game("minecraft:creative_category has been deprecated and is not available in json format {} and above. Use description->menu_category->group instead.");

/// `minecraft:seed` above 1.16.100 (a warning in the game; the key is kept).
pub static SEED_DEPRECATED: CodeDef<0> = CodeDef::new(
    "item/seed_deprecated",
    Kind::Deprecated,
    Class::Warning,
    "minecraft:seed is deprecated above 1.16.100",
)
.game("minecraft:seed has been deprecated in format versions 1.16.100 and 1.17.0 and above. Use minecraft:block_placer instead");

/// A legacy-format document whose `components` is no object.
pub static COMPONENTS_NOT_OBJECT: CodeDef<0> = CodeDef::new(
    "item/components_not_object",
    Kind::WrongType,
    Class::Error,
    "components is not an object",
)
.game("Item 'components' should be a json object");

/// An unknown effect name of a legacy food (`effect` or `remove_effect`); the arguments are
/// the item, which list and the name. The game adds the effect on: a kept error.
pub static LEGACY_EFFECT_NAME: CodeDef<3> = CodeDef::new(
    "item/legacy_effect_name",
    Kind::InvalidValue,
    Class::KeptError,
    "item {}: unknown {} name {}",
)
.game("Item[{}] Failed to load {} with name[{}] from Json. bad name?");

// ---- the JSON item path ----

/// A resource-pack item at or above 1.16.100; the argument is the identifier.
pub static ITEM_IN_RESOURCE_PACK: CodeDef<1> = CodeDef::new(
    "item/item_in_resource_pack",
    Kind::UnsupportedVersion,
    Class::Error,
    "item {} at 1.16.100 or above in a resource pack",
)
.game("Item '{}' with format_version 1.16.100 or greater cannot be loaded from resource packs.");

/// A JSON item whose root is no object. The finding carries the reader's message as its
/// `DiagnosticDetail::Syntax`.
pub static ROOT_NOT_OBJECT: CodeDef<0> = CodeDef::new(
    "item/root_not_object",
    Kind::Syntax,
    Class::Error,
    "the item's root is not an object",
)
.game("failed to parse file into a json document, root value needs to be a json object");

/// The second line the game writes after an unreadable version or a root that is no object,
/// and the only one for a JSON syntax error; the finding carries the reader's message (or the
/// crate's) as its `DiagnosticDetail::Syntax`.
pub static INVALID_JSON_OBJECT: CodeDef<0> = CodeDef::new(
    "item/invalid_json_object",
    Kind::Syntax,
    Class::Error,
    "the item is not a JSON object",
)
.game("Item is an invalid json object.");

/// The JSON path finds no readable version.
pub static VERSION_NOT_READABLE: CodeDef<0> = CodeDef::new(
    "item/version_not_readable",
    Kind::FormatVersion,
    Class::Error,
    "the item's format_version is not readable",
)
.game("document's version not found or not readable");

/// A JSON item whose version is above the game's current format version.
pub static VERSION_OUT_OF_RANGE: CodeDef<0> = CodeDef::new(
    "item/version_out_of_range",
    Kind::UnsupportedVersion,
    Class::Error,
    "the item's format_version is above the game's",
)
.game("document's version is out of handled range");

/// No `minecraft:item` object.
pub static PAYLOAD_NOT_OBJECT: CodeDef<0> = CodeDef::new(
    "item/payload_not_object",
    Kind::Envelope,
    Class::Error,
    "no minecraft:item object",
)
.game(
    "'minecraft:item' should be a json object - it has to start with an \"{\" and end with a \"}\"",
);

/// A `minecraft:item` that is no object, told at the payload. The finding carries the crate's
/// message as its `DiagnosticDetail::Syntax`.
pub static PAYLOAD_NOT_OBJECT_MALFORMED_INPUT: CodeDef<0> = CodeDef::new(
    "item/payload_not_object_malformed_input",
    Kind::Syntax,
    Class::Error,
    "minecraft:item is not an object",
)
.game(
    "'minecraft:item' should be a json object - it has to start with an \"{\" and end with a \"}\"",
);

// ---- the version history ----

/// `minecraft:armor` without `minecraft:wearable` below 1.20.20.
pub static ARMOR_WITHOUT_WEARABLE: CodeDef<0> = CodeDef::new(
    "item/armor_without_wearable",
    Kind::MissingRequired,
    Class::Error,
    "minecraft:armor without minecraft:wearable",
)
.game("Item must have a 'minecraft:wearable' component to be equippable");

/// An unknown `saturation_modifier` name of a legacy food; the argument is the name. The game
/// goes on with 0.6: a kept error.
pub static SATURATION_MODIFIER: CodeDef<1> = CodeDef::new(
    "item/saturation_modifier",
    Kind::InvalidValue,
    Class::KeptError,
    "unknown saturation_modifier {}",
)
.game("unknown saturation_modifier type '{}' in food component, defaulted to 0.6");

/// An author-written `__deprecated_components` from 1.20.30 below 1.26.30.
pub static DEPRECATED_COMPONENTS_DEPRECATED: CodeDef<0> = CodeDef::new(
    "item/deprecated_components_deprecated",
    Kind::Deprecated,
    Class::Error,
    "__deprecated_components is deprecated",
)
.game("member \"__deprecated_components\" is deprecated");

/// `deprecated_component_data` at 1.26.30.
pub static DEPRECATED_COMPONENT_DATA_DEPRECATED: CodeDef<0> = CodeDef::new(
    "item/deprecated_component_data_deprecated",
    Kind::Deprecated,
    Class::Error,
    "deprecated_component_data is deprecated",
)
.game("member \"deprecated_component_data\" is deprecated");

/// `legacy_data` at 1.26.30.
pub static LEGACY_DATA_DEPRECATED: CodeDef<0> = CodeDef::new(
    "item/legacy_data_deprecated",
    Kind::Deprecated,
    Class::Error,
    "legacy_data is deprecated",
)
.game("member \"legacy_data\" is deprecated");

/// `menu_category.group` longer than 256 characters (the crate's own sentence).
pub static GROUP_LENGTH: CodeDef<0> = CodeDef::new(
    "item/group_length",
    Kind::InvalidValue,
    Class::Error,
    "menu_category group longer than 256 characters",
)
.game("menu_category -> group: string must be at most 256 characters");

/// `menu_category.group` without a namespace, from 1.21.60.
pub static GROUP_NAMESPACE: CodeDef<0> = CodeDef::new(
    "item/group_namespace",
    Kind::Identifier,
    Class::Error,
    "menu_category group without a namespace",
)
.game("menu_category -> group: string must be prefixed with a namespace (eg. namespace:value)");

/// `minecraft:enchantable` with a slot the game does not know; the argument is the slot as
/// written.
pub static ENCHANT_SLOT: CodeDef<1> = CodeDef::new(
    "item/enchant_slot",
    Kind::InvalidValue,
    Class::Error,
    "{} is no enchant slot",
)
.game("`minecraft:enchantable`: `{}` is an invalid enchant slot, `none` has been used instead");

/// A custom component name in the `minecraft` namespace (the crate's own sentence).
pub static CUSTOM_COMPONENT_NAMESPACE: CodeDef<0> = CodeDef::new(
    "item/custom_component_namespace",
    Kind::InvalidValue,
    Class::Error,
    "a custom component name in the minecraft namespace",
)
.game("custom component names must not use the minecraft namespace");

/// A legacy-creator-features component without a release version; the argument is the
/// component (the crate's own sentence: the game has none).
pub static LEGACY_ONLY: CodeDef<1> = CodeDef::new(
    "item/legacy_only",
    Kind::Toggle,
    Class::Error,
    "item component {} is only for legacy vanilla packs",
)
.game("Item component '{}' is only available to legacy vanilla packs");

// ---- the rules ----

/// The identifier rule.
pub static IDENTIFIER: CodeDef<0> = CodeDef::new(
    "item/identifier",
    Kind::Identifier,
    Class::Error,
    "the item identifier breaks the namespace rule",
)
.game("must include a namespace and must not use the Minecraft namespace unless overriding a Vanilla item");

/// A component that needs `minecraft:use_modifiers > use_duration` (a warning in the game); the
/// argument is the component.
pub static USE_DURATION_REQUIRED: CodeDef<1> = CodeDef::new(
    "item/use_duration_required",
    Kind::MissingRequired,
    Class::Warning,
    "{} without a non-zero use_duration",
)
.game("The \"{}\" component requires \"minecraft:use_modifiers > use_duration\" to be set with a non-zero value.");

/// A component that needs a `minecraft:durability` with a positive `max_durability` (a warning
/// in the game); the argument is the component.
pub static DURABILITY_REQUIRED: CodeDef<1> = CodeDef::new(
    "item/durability_required",
    Kind::MissingRequired,
    Class::Warning,
    "{} without a positive max_durability",
)
.game("The \"{}\" component requires a \"minecraft:durability\" component with positive \"max_durability\"");

/// `minecraft:bundle_interaction` without `minecraft:storage_item` (a warning in the game).
pub static BUNDLE_NEEDS_STORAGE: CodeDef<0> = CodeDef::new(
    "item/bundle_needs_storage",
    Kind::MissingRequired,
    Class::Warning,
    "minecraft:bundle_interaction without minecraft:storage_item",
)
.game("The \"minecraft:bundle_interaction\" component requires a \"minecraft:storage_item\" component.");

/// `minecraft:storage_item` without a stack size of 1.
pub static STORAGE_NEEDS_STACK_OF_ONE: CodeDef<0> = CodeDef::new(
    "item/storage_needs_stack_of_one",
    Kind::InvalidValue,
    Class::Error,
    "minecraft:storage_item without a stack size of 1",
)
.game("The \"minecraft:storage_item\" component requires a \"minecraft:max_stack_size\" component set to 1.");

/// `minecraft:swing_sounds` without a member.
pub static SWING_SOUNDS_EMPTY: CodeDef<0> = CodeDef::new(
    "item/swing_sounds_empty",
    Kind::MissingRequired,
    Class::Error,
    "minecraft:swing_sounds without a member",
)
.game("\"minecraft:swing_sounds\" requires at least one of its fields to be defined.");

/// A wearable without `minecraft:max_stack_size` from 1.26.30; the arguments are the component
/// and the slot. The item loads: a kept error.
pub static WEARABLE_STACK_SIZE: CodeDef<2> = CodeDef::new(
    "item/wearable_stack_size",
    Kind::InvalidValue,
    Class::KeptError,
    "{} in slot {} defaults to a stack of 1",
)
.game("Items with the '{}' component and 'slot' set to '{}' will default to a max stack size of 1. Use the 'minecraft:max_stack_size' component to override.");

/// `minecraft:kinetic_weapon` without `minecraft:use_modifiers`.
pub static KINETIC_NEEDS_USE_MODIFIERS: CodeDef<0> = CodeDef::new(
    "item/kinetic_needs_use_modifiers",
    Kind::MissingRequired,
    Class::Error,
    "minecraft:kinetic_weapon without minecraft:use_modifiers",
)
.game(
    "The \"minecraft:kinetic_weapon\" component requires a \"minecraft:use_modifiers\" component.",
);

/// `minecraft:kinetic_weapon` without a condition.
pub static KINETIC_NEEDS_CONDITION: CodeDef<0> = CodeDef::new(
    "item/kinetic_needs_condition",
    Kind::MissingRequired,
    Class::Error,
    "minecraft:kinetic_weapon without a condition",
)
.game("\"minecraft:kinetic_weapon\" requires at least one of \"damage_conditions\", \"knockback_conditions\", or \"dismount_conditions\" to be defined.");

/// A kinetic condition with a `max_duration` of zero; the arguments are the condition member,
/// twice (the game names it twice).
pub static KINETIC_ZERO_DURATION: CodeDef<2> = CodeDef::new(
    "item/kinetic_zero_duration",
    Kind::InvalidValue,
    Class::Error,
    "{} of minecraft:kinetic_weapon has a max_duration of zero ({})",
)
.game("\"minecraft:kinetic_weapon\"'s \"{}\" cannot have a \"max_duration\" of zero. To disable the effect, leave the \"{}\" field undefined.");

/// A kinetic condition with both speeds; the argument is the condition member.
pub static KINETIC_SPEED_EXCLUSIVE: CodeDef<1> = CodeDef::new(
    "item/kinetic_speed_exclusive",
    Kind::InvalidValue,
    Class::Error,
    "{} of minecraft:kinetic_weapon has both min_speed and min_relative_speed",
)
.game("Within \"minecraft:kinetic_weapon\"'s \"{}\", \"min_speed\" and \"min_relative_speed\" are mutually exclusive. Specify only one.");

// ---- the combine ----

/// An item a higher pack already defined; the argument is the identifier. The game skips it with
/// a warning.
pub static OVERRIDDEN_BY_A_HIGHER_PACK: CodeDef<1> = CodeDef::new(
    "item/overridden_by_a_higher_pack",
    Kind::Shadowed,
    Class::Warning,
    "item {} already defined by a higher pack",
)
.game("Item '{}' has already been overridden by a pack higher in the pack stack!");

/// A legacy item over a vanilla item that exists only at 1.16.100 or above; the arguments are the
/// identifier and the version as written.
pub static VANILLA_IS_COMPONENT_BASED: CodeDef<2> = CodeDef::new(
    "item/vanilla_is_component_based",
    Kind::Identifier,
    Class::Error,
    "item {} ({}) over a component-based vanilla item",
)
.game("Item '{}' (format_version: {}) cannot be registered because the Vanilla item only exists as a 1.16.100+ format version!");

/// A data-driven item over a vanilla item that has no 1.16.100+ version (or none at all); the
/// arguments are the identifier and the version as written.
pub static VANILLA_IS_LEGACY: CodeDef<2> = CodeDef::new(
    "item/vanilla_is_legacy",
    Kind::Identifier,
    Class::Error,
    "item {} ({}) over no component-based vanilla item",
)
.game("Item '{}' (format_version: {}) cannot be registered because no 1.16.100+ Vanilla format version exists!");

/// A legacy item in the `minecraft` namespace that is no vanilla item; the argument is the
/// identifier.
pub static NON_VANILLA_NAMESPACE: CodeDef<1> = CodeDef::new(
    "item/non_vanilla_namespace",
    Kind::Identifier,
    Class::Error,
    "item {} is no vanilla item and uses the minecraft namespace",
)
.game("Non-Vanilla item '{}' should not be using the \"minecraft\" namespace!");

// ---- the trim registries ----

/// A received pattern whose template item an earlier entry already named; the argument is the
/// item. The game goes on to the next entry: a kept error.
pub static DUPLICATE_TEMPLATE: CodeDef<1> = CodeDef::new(
    "item/duplicate_template",
    Kind::Duplicate,
    Class::KeptError,
    "pattern template item {} named twice",
)
.game("Overwriting pattern template. Duplicate entries with template item: \"{}\"");

/// Every definition of this table, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    FOIL_DEPRECATED.meta(),
    FOIL_IGNORED.meta(),
    MINING_SPEED_DEPRECATED.meta(),
    TAG_DEPRECATED.meta(),
    CREATIVE_CATEGORY_DEPRECATED.meta(),
    SEED_DEPRECATED.meta(),
    COMPONENTS_NOT_OBJECT.meta(),
    LEGACY_EFFECT_NAME.meta(),
    ITEM_IN_RESOURCE_PACK.meta(),
    ROOT_NOT_OBJECT.meta(),
    INVALID_JSON_OBJECT.meta(),
    VERSION_NOT_READABLE.meta(),
    VERSION_OUT_OF_RANGE.meta(),
    PAYLOAD_NOT_OBJECT.meta(),
    PAYLOAD_NOT_OBJECT_MALFORMED_INPUT.meta(),
    ARMOR_WITHOUT_WEARABLE.meta(),
    SATURATION_MODIFIER.meta(),
    DEPRECATED_COMPONENTS_DEPRECATED.meta(),
    DEPRECATED_COMPONENT_DATA_DEPRECATED.meta(),
    LEGACY_DATA_DEPRECATED.meta(),
    GROUP_LENGTH.meta(),
    GROUP_NAMESPACE.meta(),
    ENCHANT_SLOT.meta(),
    CUSTOM_COMPONENT_NAMESPACE.meta(),
    LEGACY_ONLY.meta(),
    IDENTIFIER.meta(),
    USE_DURATION_REQUIRED.meta(),
    DURABILITY_REQUIRED.meta(),
    BUNDLE_NEEDS_STORAGE.meta(),
    STORAGE_NEEDS_STACK_OF_ONE.meta(),
    SWING_SOUNDS_EMPTY.meta(),
    WEARABLE_STACK_SIZE.meta(),
    KINETIC_NEEDS_USE_MODIFIERS.meta(),
    KINETIC_NEEDS_CONDITION.meta(),
    KINETIC_ZERO_DURATION.meta(),
    KINETIC_SPEED_EXCLUSIVE.meta(),
    OVERRIDDEN_BY_A_HIGHER_PACK.meta(),
    VANILLA_IS_COMPONENT_BASED.meta(),
    VANILLA_IS_LEGACY.meta(),
    NON_VANILLA_NAMESPACE.meta(),
    DUPLICATE_TEMPLATE.meta(),
];
