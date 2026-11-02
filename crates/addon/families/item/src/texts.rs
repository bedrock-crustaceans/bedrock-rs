//! The item family's fixed content-log texts (`VanillaText::Fixed`). Texts marked "the
//! crate's own" are worded by this crate: the game says nothing for those cases, or its text
//! for them is not known.

/// `minecraft:foil` at or above 1.20.20.
pub(crate) const FOIL_DEPRECATED: &str = "minecraft:foil has been deprecated in format versions 1.20.20 and above. Use minecraft:glint instead.";
/// `minecraft:foil` below 1.20.20 in a pack without the legacy creator features (the
/// crate's own text).
pub(crate) const FOIL_IGNORED: &str =
    "minecraft:foil is ignored for this pack; use minecraft:glint";
/// `minecraft:mining_speed` at or above 1.20.30.
pub(crate) const MINING_SPEED_DEPRECATED: &str = "minecraft:mining_speed has been deprecated in format versions 1.20.30 and above. Use minecraft:digger instead.";
/// A `tag:` key at or above 1.20.50.
pub(crate) const TAG_DEPRECATED: &str = "tag: has been deprecated in format versions 1.20.50 and above. Use the minecraft:tags component to store all tags instead.";
/// An item a higher pack already defined; the game logs it as a warning.
pub(crate) const OVERRIDDEN_BY_A_HIGHER_PACK: &str =
    "Item '{}' has already been overridden by a pack higher in the pack stack!";
/// `minecraft:seed` above 1.16.100.
pub(crate) const SEED_DEPRECATED: &str = "minecraft:seed has been deprecated in format versions 1.16.100 and 1.17.0 and above. Use minecraft:block_placer instead";
/// The list form of `minecraft:custom_components` at or above 1.21.90.
pub(crate) const CUSTOM_COMPONENTS_DEPRECATED: &str =
    "member \"minecraft:custom_components\" is deprecated";
/// An author-written `__deprecated_components` below 1.26.30.
pub(crate) const DEPRECATED_COMPONENTS_DEPRECATED: &str =
    "member \"__deprecated_components\" is deprecated";
/// `deprecated_component_data` at 1.26.30.
pub(crate) const DEPRECATED_COMPONENT_DATA_DEPRECATED: &str =
    "member \"deprecated_component_data\" is deprecated";
/// `legacy_data` at 1.26.30.
pub(crate) const LEGACY_DATA_DEPRECATED: &str = "member \"legacy_data\" is deprecated";
/// A custom component name in the `minecraft` namespace (the crate's own text).
pub(crate) const CUSTOM_COMPONENT_NAMESPACE: &str =
    "custom component names must not use the minecraft namespace";
/// A legacy-format document whose `components` is no object.
pub(crate) const COMPONENTS_NOT_OBJECT: &str = "Item 'components' should be a json object";
/// `minecraft:armor` without `minecraft:wearable` below 1.20.20.
pub(crate) const ARMOR_WITHOUT_WEARABLE: &str =
    "Item must have a 'minecraft:wearable' component to be equippable";
/// The identifier rule.
pub(crate) const IDENTIFIER: &str = "must include a namespace and must not use the Minecraft namespace unless overriding a Vanilla item";
/// `menu_category.group` without a namespace, from 1.21.60.
pub(crate) const GROUP_NAMESPACE: &str =
    "menu_category -> group: string must be prefixed with a namespace (eg. namespace:value)";
/// `menu_category.group` longer than 256 characters (the crate's own text).
pub(crate) const GROUP_LENGTH: &str =
    "menu_category -> group: string must be at most 256 characters";
/// `minecraft:bundle_interaction` without `minecraft:storage_item`.
pub(crate) const BUNDLE_NEEDS_STORAGE: &str = "The \"minecraft:bundle_interaction\" component requires a \"minecraft:storage_item\" component.";
/// `minecraft:storage_item` without a stack size of 1.
pub(crate) const STORAGE_NEEDS_STACK_OF_ONE: &str = "The \"minecraft:storage_item\" component requires a \"minecraft:max_stack_size\" component set to 1.";
/// `minecraft:kinetic_weapon` without `minecraft:use_modifiers`.
pub(crate) const KINETIC_NEEDS_USE_MODIFIERS: &str =
    "The \"minecraft:kinetic_weapon\" component requires a \"minecraft:use_modifiers\" component.";
/// `minecraft:kinetic_weapon` without a condition.
pub(crate) const KINETIC_NEEDS_CONDITION: &str = "\"minecraft:kinetic_weapon\" requires at least one of \"damage_conditions\", \"knockback_conditions\", or \"dismount_conditions\" to be defined.";
/// `minecraft:swing_sounds` without a member.
pub(crate) const SWING_SOUNDS_EMPTY: &str =
    "\"minecraft:swing_sounds\" requires at least one of its fields to be defined.";
/// A member the strict 1.26.50 format does not bind, held from an older format (the crate's
/// own text). The newest format refuses a document with such a member, so the item cannot be
/// written as it is.
pub(crate) const NOT_IN_THE_NEWEST_FORMAT: &str = "the newest format (1.26.50) does not bind this member and refuses a document with it: the item is written as a 1.26.0 document when that text reads back as the item, else not written";
/// A root `format_version` written as a number (the crate's own text). The game re-reads
/// the version of a JSON item from its text when it is no string.
pub(crate) const NUMERIC_VERSION: &str =
    "the version written as a number is read from its text; it is written back as a string";
/// The JSON path finds no readable version.
pub(crate) const VERSION_NOT_READABLE: &str = "document's version not found or not readable";
/// The second line the game writes after an unreadable version or a root that is no
/// object, and the only one for a JSON syntax error.
pub(crate) const INVALID_JSON_OBJECT: &str = "Item is an invalid json object.";
/// A JSON item whose version is above the game's current format version.
pub(crate) const VERSION_OUT_OF_RANGE: &str = "document's version is out of handled range";
/// A JSON item whose root is no object.
pub(crate) const ROOT_NOT_OBJECT: &str =
    "failed to parse file into a json document, root value needs to be a json object";
/// No `minecraft:item` object.
pub(crate) const PAYLOAD_NOT_OBJECT: &str =
    "'minecraft:item' should be a json object - it has to start with an \"{\" and end with a \"}\"";
