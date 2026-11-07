//! The crafting item catalogue's codes: vanilla's content-log lines for
//! `item_catalog/crafting_item_catalog.json`, byte for byte. Each id is `crafting_catalog/<name>`,
//! the name its constant's in lower case.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

// ---- the loader's checks before the versioned parse ----

/// A `format_version` below 1.21.60: vanilla's text of a file without a valid version
/// ([`VERSION_NOT_PROCESSABLE`](bedrock_addon_core::diagnostic::codes::VERSION_NOT_PROCESSABLE)).
pub static BELOW_FLOOR: CodeDef<0> = CodeDef::new(
    "crafting_catalog/below_floor",
    Kind::UnsupportedVersion,
    Class::Error,
    "format_version below 1.21.60",
)
.game("The file cannot be processed without a valid format_version");

/// A document whose payload member is not the catalogue's.
pub static NO_CATALOG_PAYLOAD: CodeDef<0> = CodeDef::new(
    "crafting_catalog/no_catalog_payload",
    Kind::MissingRequired,
    Class::Error,
    "no minecraft:crafting_items_catalog object",
)
.game("The crafting item catalog needs to have an object minecraft:crafting_items_catalog");

// ---- the reader ----

/// A group name that fails its namespace constraint.
pub static GROUP_NAME: CodeDef<0> = CodeDef::new(
    "crafting_catalog/group_name",
    Kind::InvalidValue,
    Class::Error,
    "the group name has no namespace",
)
.game("string must be prefixed with a namespace (eg. namespace:value)");

// ---- the merge ----

/// An item name with a bad aux value: the item lookup gives no item.
pub static INVALID_AUX: CodeDef<1> = CodeDef::new(
    "crafting_catalog/invalid_aux",
    Kind::InvalidValue,
    Class::Error,
    "invalid aux value in the item {}",
)
.game("Invalid aux value in item ID \"{}\"");

/// The linked-asset validator's line for a catalogue item no definition registers: the
/// breadcrumb of the item, then its name.
pub static MISSING_ITEM: CodeDef<2> = CodeDef::new(
    "crafting_catalog/missing_item",
    Kind::UnresolvedReference,
    Class::Error,
    "{}: missing referenced item {}",
)
.game("{}:  Missing referenced asset {}");

/// An item and aux value placed a second time: the first placement wins.
pub static DUPLICATE_ITEM: CodeDef<1> = CodeDef::new(
    "crafting_catalog/duplicate_item",
    Kind::Duplicate,
    Class::Error,
    "the item {} is in the catalog twice",
)
.game("You can only add an item to the crafting item catalog once. Found duplicate {}");

// ---- the link pass ----

/// An item whose own definition names another creative category than the catalogue's; the
/// arguments are the item, its own category and the catalogue's.
pub static CATEGORY_MISMATCH: CodeDef<3> = CodeDef::new(
    "crafting_catalog/category_mismatch",
    Kind::InvalidValue,
    Class::Warning,
    "the item {} of the category {} is placed in the category {}",
)
.game("The item {} was created with the category set to '{}', and is now being set to '{}'");

/// An item whose own definition names another group than the catalogue's; the arguments are
/// the item, its own group and the catalogue's.
pub static GROUP_MISMATCH: CodeDef<3> = CodeDef::new(
    "crafting_catalog/group_mismatch",
    Kind::InvalidValue,
    Class::Warning,
    "the item {} of the group {} is placed in the group {}",
)
.game("The item {} was created with the group set to '{}', but is now being set to '{}'");

/// Every definition of this table, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    BELOW_FLOOR.meta(),
    NO_CATALOG_PAYLOAD.meta(),
    GROUP_NAME.meta(),
    INVALID_AUX.meta(),
    MISSING_ITEM.meta(),
    DUPLICATE_ITEM.meta(),
    CATEGORY_MISMATCH.meta(),
    GROUP_MISMATCH.meta(),
];
