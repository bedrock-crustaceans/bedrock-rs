//! The loot family's codes: vanilla's content-log lines of the loot walkers, byte for byte with
//! `%s` / `%d` written `{}`, each a definition beside the walker that raises it. Each
//! [`was`](CodeDef::was) the legacy code its sites raised, so a pin or a filter on the code's
//! name keeps matching. The arity is the definition's own, so a finding cannot be built with
//! the wrong number of arguments.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Disposition, Kind};

/// A file the JSON reader rejects, or whose root is not an object; the argument is its path. The
/// finding carries the reader's message as its `DiagnosticDetail::Syntax`.
pub static INVALID_TABLE: CodeDef<1> = CodeDef::new(
    "loot/invalid_table",
    Kind::Syntax,
    Class::Error,
    "the loot table {} is not a JSON object",
)
.game("Loot table for {} is invalid.")
.was("MalformedInput");

/// A file of no value at all (no bytes, a byte-order mark alone, whitespace and comments,
/// `null`); the argument is its path.
pub static INVALID_TABLE_EMPTY: CodeDef<1> = CodeDef::new(
    "loot/invalid_table_empty",
    Kind::Syntax,
    Class::Error,
    "the loot table {} is empty",
)
.game("Loot table for {} is invalid.")
.was("EmptyDocument");

/// A pool without `entries`; the pool is kept empty: a kept error.
pub static ENTRIES_MISSING: CodeDef<0> = CodeDef::new(
    "loot/entries_missing",
    Kind::MissingRequired,
    Class::KeptError,
    "a pool without entries",
)
.game("There must be an \"entries\" array inside the loot table pool.")
.was("MissingRequiredField");

/// A pool's `entries` that is not an array; the pool is kept empty: a kept error.
pub static ENTRIES_ARRAY: CodeDef<0> = CodeDef::new(
    "loot/entries_array",
    Kind::WrongType,
    Class::KeptError,
    "a pool's entries is not an array",
)
.game("There must be an \"entries\" array inside the loot table pool.")
.was("InvalidType");

/// An entry that is no object; it is dropped: a kept error.
pub static ENTRY_OBJECT: CodeDef<0> = CodeDef::new(
    "loot/entry_object",
    Kind::WrongType,
    Class::KeptError,
    "an entry is not an object",
)
.game("Every entry of \"entries\" must be a JSON object - it has to start with an \"{\" and end with a \"}\"")
.was("InvalidType");

/// An entry of no known type; the argument is the type as written. It is dropped: a kept
/// error.
pub static ENTRY_TYPE: CodeDef<1> = CodeDef::new(
    "loot/entry_type",
    Kind::InvalidValue,
    Class::KeptError,
    "an entry of the unknown type {}",
)
.game("Every entry of \"entries\" must specify a type of either \"item\", \"loot_table\", or \"empty\". Given: {}")
.was("InvalidValue");

/// A function that is no object; it is dropped: a kept error.
pub static FUNCTION_OBJECT: CodeDef<0> = CodeDef::new(
    "loot/function_object",
    Kind::WrongType,
    Class::KeptError,
    "a function is not an object",
)
.game("Error loading loot table \"functions\" as it is not a valid Json object. \"functions\" must be key: value pairs")
.was("InvalidType");

/// A function the game does not know; the argument is its name. It is dropped: a kept error.
pub static UNSUPPORTED_FUNCTION: CodeDef<1> = CodeDef::new(
    "loot/unsupported_function",
    Kind::UnknownMember(Disposition::Kept),
    Class::KeptError,
    "unknown loot function {}",
)
.game("Unsupported looting function: {}")
.was("UnrecognizedComponent");

/// A function of the Upcoming Creator Features experiment without the toggle; the argument is
/// its name. It is dropped: a kept error.
pub static UCF_FUNCTION: CodeDef<1> = CodeDef::new(
    "loot/ucf_function",
    Kind::Toggle,
    Class::KeptError,
    "loot function {} needs the Upcoming Creator Features toggle",
)
.game("Unsupported looting function outside of Upcoming Creator Features Experiment: {}")
.was("MissingToggles");

/// `carry_over_block_entity_data`'s `dynamic_properties` that is no boolean: a kept error.
pub static DYNAMIC_PROPERTIES: CodeDef<0> = CodeDef::new(
    "loot/dynamic_properties",
    Kind::WrongType,
    Class::KeptError,
    "dynamic_properties is not a boolean",
)
.game("Error loading loot table \"carry_over_block_entity_data\" dynamic_properties as it is not a bool")
.was("InvalidType");

/// A condition that is no object; it is dropped: a kept error.
pub static CONDITION_OBJECT: CodeDef<0> = CodeDef::new(
    "loot/condition_object",
    Kind::WrongType,
    Class::KeptError,
    "a condition is not an object",
)
.game("Error loading loot table \"conditions\" as it is not a valid Json object. \"conditions\" must be key: value pairs")
.was("InvalidType");

/// A condition's operator that is none of the four; the argument is the operator as written.
/// The condition is dropped: a kept error.
pub static CONDITION_OPERATOR: CodeDef<1> = CodeDef::new(
    "loot/condition_operator",
    Kind::InvalidValue,
    Class::KeptError,
    "unknown loot condition operator {}",
)
.game("Invalid loot condition operator \"{}\". Expected \"==\", \"equals\", \"!=\", or \"not\".")
.was("InvalidValue");

/// A condition the game does not know; the argument is its name. The game writes the line to
/// its developer log, not the content log: an information.
pub static UNKNOWN_CONDITION: CodeDef<1> = CodeDef::new(
    "loot/unknown_condition",
    Kind::UnknownMember(Disposition::Kept),
    Class::Lint,
    "unknown loot condition {}",
)
.game("There is unknown or supressed condition[{}]")
.was("UnrecognizedComponent");

/// A banner `type` the game does not know; the argument is the value as written: a kept error.
pub static BANNER_TYPE: CodeDef<1> = CodeDef::new(
    "loot/banner_type",
    Kind::InvalidValue,
    Class::KeptError,
    "unknown banner type {}",
)
.game("Invalid banner type input: {}")
.was("InvalidValue");

/// A banner `base_color` the game does not know; the argument is the value as written: a kept
/// error.
pub static BANNER_BASE_COLOR: CodeDef<1> = CodeDef::new(
    "loot/banner_base_color",
    Kind::InvalidValue,
    Class::KeptError,
    "unknown banner base_color {}",
)
.game("Invalid banner base_color input: {}")
.was("InvalidValue");

/// Patterns on a banner type other than the default: a kept error.
pub static BANNER_NON_DEFAULT: CodeDef<0> = CodeDef::new(
    "loot/banner_non_default",
    Kind::InvalidValue,
    Class::KeptError,
    "patterns on a banner that is not of the default type",
)
.game("Non-default banner types cannot have custom patterns")
.was("InvalidValue");

/// More patterns than a banner takes; the arguments are the limit and the number given: a
/// kept error.
pub static BANNER_MAX: CodeDef<2> = CodeDef::new(
    "loot/banner_max",
    Kind::InvalidValue,
    Class::KeptError,
    "more than {} banner patterns ({})",
)
.game("Banner exceeds maximum number of {} patterns! Patterns given: {}")
.was("InvalidValue");

/// A banner pattern the game does not know; the argument is its name (the game prints none):
/// a kept error.
pub static BANNER_PATTERN: CodeDef<1> = CodeDef::new(
    "loot/banner_pattern",
    Kind::UnresolvedReference,
    Class::KeptError,
    "unknown banner pattern {}",
)
.game("Invalid banner pattern name: {}")
.was("UnresolvedReference");

/// A banner pattern's colour the game does not know; the argument is the value as written: a
/// kept error.
pub static BANNER_PATTERN_COLOR: CodeDef<1> = CodeDef::new(
    "loot/banner_pattern_color",
    Kind::InvalidValue,
    Class::KeptError,
    "unknown colour {} of a banner pattern",
)
.game("Invalid color input in banner pattern {}")
.was("InvalidValue");

/// `set_data` with neither `data` nor `values` (a warning in the game).
pub static SET_DATA_KEY: CodeDef<0> = CodeDef::new(
    "loot/set_data_key",
    Kind::MissingRequired,
    Class::Warning,
    "set_data without data or values",
)
.game("Invalid json keyname for specifying the data range for set_data function. Only 'data' and 'values' are supported.")
.was("MissingRequiredField");

/// `enchant_book_for_trading`'s `enchantments` that is no array: a kept error.
pub static ENCHANT_ARRAY: CodeDef<0> = CodeDef::new(
    "loot/enchant_array",
    Kind::WrongType,
    Class::KeptError,
    "enchantments is not an array",
)
.game("Error loading \"enchant_book_for_trading\" function, \"enchantments\": should be type array")
.was("InvalidType");

/// An `enchant_book_for_trading` enchantment the game does not know; the argument is its name
/// as written. It is dropped: a kept error.
pub static ENCHANT_NAME: CodeDef<1> = CodeDef::new(
    "loot/enchant_name",
    Kind::UnresolvedReference,
    Class::KeptError,
    "unknown enchantment {}",
)
.game("Error loading \"enchant_book_for_trading\" function, \"name\": {} is not a valid enchantment name")
.was("UnresolvedReference");

/// Vanilla's line for a loot entry's item name the game does not have, with `%s` written `{}`.
/// The link pass raises it (`addon/src/pack/link/content/loot_item.rs`), which resolves item
/// names, as the sentence of its unresolved-reference finding; the family's walkers do not.
pub const INVALID_ITEM: &str = "{} is not a valid item name.";

/// Every definition of this table, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    INVALID_TABLE.meta(),
    INVALID_TABLE_EMPTY.meta(),
    ENTRIES_MISSING.meta(),
    ENTRIES_ARRAY.meta(),
    ENTRY_OBJECT.meta(),
    ENTRY_TYPE.meta(),
    FUNCTION_OBJECT.meta(),
    UNSUPPORTED_FUNCTION.meta(),
    UCF_FUNCTION.meta(),
    DYNAMIC_PROPERTIES.meta(),
    CONDITION_OBJECT.meta(),
    CONDITION_OPERATOR.meta(),
    UNKNOWN_CONDITION.meta(),
    BANNER_TYPE.meta(),
    BANNER_BASE_COLOR.meta(),
    BANNER_NON_DEFAULT.meta(),
    BANNER_MAX.meta(),
    BANNER_PATTERN.meta(),
    BANNER_PATTERN_COLOR.meta(),
    SET_DATA_KEY.meta(),
    ENCHANT_ARRAY.meta(),
    ENCHANT_NAME.meta(),
];

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::diagnostic::sentence;

    #[test]
    fn the_invalid_item_line_fills_its_placeholder() {
        assert_eq!(INVALID_ITEM.matches("{}").count(), 1);
        assert_eq!(
            sentence(INVALID_ITEM, &["a:b"]),
            "a:b is not a valid item name."
        );
    }
}
