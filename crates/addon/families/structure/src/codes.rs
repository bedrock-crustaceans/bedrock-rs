//! The structure family's codes: vanilla's content-log lines of the structure loader, byte for
//! byte (`text.rs` fills their arguments). Each [`was`](CodeDef::was) the legacy code its
//! sites raised, so a pin or a filter on the code's name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

// ---- the root ----

/// Bytes that are not an NBT compound: the game reads an empty root, which fails the
/// `format_version` step (the argument is `format_version`). The finding carries the reader's
/// message as its `DiagnosticDetail::Syntax`.
pub static NOT_NBT: CodeDef<1> = CodeDef::new(
    "structure/not_nbt",
    Kind::Syntax,
    Class::Error,
    "the file is not an NBT compound",
)
.game("\"{}\" field, a required field, is missing from the structure.")
.was("MalformedInput");

/// `"%s" field, a required field, is missing from the structure.`
pub static REQUIRED: CodeDef<1> = CodeDef::new(
    "structure/required",
    Kind::MissingRequired,
    Class::Error,
    "the required field {} is missing",
)
.game("\"{}\" field, a required field, is missing from the structure.")
.was("MissingRequiredField");

/// `"%s" field has a value larger than the current version.  We expect a format version with
/// a value at most %d and got %d.` (two spaces after "version.").
pub static LARGER_VERSION: CodeDef<3> = CodeDef::new(
    "structure/larger_version",
    Kind::UnsupportedVersion,
    Class::Error,
    "{} is above the current version {} (got {})",
)
.game("\"{}\" field has a value larger than the current version.  We expect a format version with a value at most {} and got {}.")
.was("UnsupportedVersion");

/// `"%s" field has a value smaller than the current version.  We expect a format version with
/// a value at most %d and got %d.`
pub static SMALLER_VERSION: CodeDef<3> = CodeDef::new(
    "structure/smaller_version",
    Kind::UnsupportedVersion,
    Class::Error,
    "{} is below the current version {} (got {})",
)
.game("\"{}\" field has a value smaller than the current version.  We expect a format version with a value at most {} and got {}.")
.was("UnsupportedVersion");

/// `"%s" field is not an integer.`
pub static NOT_AN_INTEGER: CodeDef<1> = CodeDef::new(
    "structure/not_an_integer",
    Kind::InvalidValue,
    Class::Error,
    "{} is not an integer",
)
.game("\"{}\" field is not an integer.")
.was("InvalidValue");

/// `"%s" field has an invalid value of %i. Valid values are %i and up to %i.`
pub static INVALID_VALUE: CodeDef<4> = CodeDef::new(
    "structure/invalid_value",
    Kind::InvalidValue,
    Class::Error,
    "{} has the invalid value {} (valid: {} to {})",
)
.game("\"{}\" field has an invalid value of {}. Valid values are {} and up to {}.")
.was("InvalidValue");

/// `The "%s" field expects 3 elements.`
pub static THREE_ELEMENTS: CodeDef<1> = CodeDef::new(
    "structure/three_elements",
    Kind::InvalidValue,
    Class::Error,
    "{} does not have 3 elements",
)
.game("The \"{}\" field expects 3 elements.")
.was("InvalidValue");

/// `The "%s" field shouldn't have any negative values.`
pub static NEGATIVE: CodeDef<1> = CodeDef::new(
    "structure/negative",
    Kind::InvalidValue,
    Class::Error,
    "{} has a negative value",
)
.game("The \"{}\" field shouldn't have any negative values.")
.was("InvalidValue");

// ---- the block indices ----

/// `The "%s" field should be an array with 2 arrays and instead we have %d arrays.`
pub static TWO_ARRAYS: CodeDef<2> = CodeDef::new(
    "structure/two_arrays",
    Kind::InvalidValue,
    Class::Error,
    "{} holds {} arrays, not 2",
)
.game("The \"{}\" field should be an array with 2 arrays and instead we have {} arrays.")
.was("InvalidValue");

/// `The "%s" field's first array is either missing or not a list.`
pub static FIRST_ARRAY: CodeDef<1> = CodeDef::new(
    "structure/first_array",
    Kind::InvalidValue,
    Class::Error,
    "the first array of {} is missing or not a list",
)
.game("The \"{}\" field's first array is either missing or not a list.")
.was("InvalidValue");

/// `The "%s" field's arrays need to both be the same size.`
pub static SAME_SIZE: CodeDef<1> = CodeDef::new(
    "structure/same_size",
    Kind::InvalidValue,
    Class::Error,
    "the arrays of {} differ in size",
)
.game("The \"{}\" field's arrays need to both be the same size.")
.was("InvalidValue");

/// `The "%s" field should have as many elements as defined by the "%s" field.`
pub static VOLUME: CodeDef<2> = CodeDef::new(
    "structure/volume",
    Kind::InvalidValue,
    Class::Error,
    "{} does not hold one element per position of {}",
)
.game("The \"{}\" field should have as many elements as defined by the \"{}\" field.")
.was("InvalidValue");

// ---- the palettes, positions and entities ----

/// `Expected a compound tag when parsing elements of "%s" field.`
pub static COMPOUND_ELEMENTS: CodeDef<1> = CodeDef::new(
    "structure/compound_elements",
    Kind::InvalidValue,
    Class::Error,
    "an element of {} is not a compound",
)
.game("Expected a compound tag when parsing elements of \"{}\" field.")
.was("InvalidValue");

/// `At index %d while reading "%s", we found an invalid block in the palette, we were
/// expecting a compound tag here.` (the field is `block_palette`)
pub static INVALID_BLOCK: CodeDef<2> = CodeDef::new(
    "structure/invalid_block",
    Kind::InvalidValue,
    Class::Error,
    "the palette entry {} of {} is not a compound",
)
.game("At index {} while reading \"{}\", we found an invalid block in the palette, we were expecting a compound tag here.")
.was("InvalidValue");

/// `Expected a string formatted as a number when parsing contents of the "%s" field, but got
/// "%s"` (no final period)
pub static NOT_A_NUMBER: CodeDef<2> = CodeDef::new(
    "structure/not_a_number",
    Kind::InvalidValue,
    Class::Error,
    "a key of {} is not a number: {}",
)
.game("Expected a string formatted as a number when parsing contents of the \"{}\" field, but got \"{}\"")
.was("InvalidValue");

/// `Expected a compound tag when parsing elements of "%s" field at the value for "%s".`
pub static COMPOUND_VALUE: CodeDef<2> = CodeDef::new(
    "structure/compound_value",
    Kind::InvalidValue,
    Class::Error,
    "the value of {} at {} is not a compound",
)
.game("Expected a compound tag when parsing elements of \"{}\" field at the value for \"{}\".")
.was("InvalidValue");

/// `Expected a compound tag for the "%s" field.`
pub static COMPOUND_FIELD: CodeDef<1> = CodeDef::new(
    "structure/compound_field",
    Kind::InvalidValue,
    Class::Error,
    "{} is not a compound",
)
.game("Expected a compound tag for the \"{}\" field.")
.was("InvalidValue");

/// `Expected a list tag for the "%s" field.`
pub static LIST_FIELD: CodeDef<1> = CodeDef::new(
    "structure/list_field",
    Kind::InvalidValue,
    Class::Error,
    "{} is not a list",
)
.game("Expected a list tag for the \"{}\" field.")
.was("InvalidValue");

/// `Expected a compound tag at index %d when parsing elements of "%s" field.`
pub static COMPOUND_AT: CodeDef<2> = CodeDef::new(
    "structure/compound_at",
    Kind::InvalidValue,
    Class::Error,
    "the element {} of {} is not a compound",
)
.game("Expected a compound tag at index {} when parsing elements of \"{}\" field.")
.was("InvalidValue");

/// `At index %d while reading "%s", we found an invalid entity in the list of entities.`
/// (the field is `entities`)
pub static INVALID_ENTITY: CodeDef<2> = CodeDef::new(
    "structure/invalid_entity",
    Kind::InvalidValue,
    Class::Error,
    "the entity {} of {} is not a compound",
)
.game("At index {} while reading \"{}\", we found an invalid entity in the list of entities.")
.was("InvalidValue");

// ---- the lookup ----

/// `Structure '%s' must be prefixed by a namespace`, written when a structure is looked up.
pub static NOT_NAMESPACED: CodeDef<1> = CodeDef::new(
    "structure/not_namespaced",
    Kind::Identifier,
    Class::Error,
    "the structure {} has no namespace",
)
.game("Structure '{}' must be prefixed by a namespace")
.was("NamespaceRule");

/// Every definition of this table, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    NOT_NBT.meta(),
    REQUIRED.meta(),
    LARGER_VERSION.meta(),
    SMALLER_VERSION.meta(),
    NOT_AN_INTEGER.meta(),
    INVALID_VALUE.meta(),
    THREE_ELEMENTS.meta(),
    NEGATIVE.meta(),
    TWO_ARRAYS.meta(),
    FIRST_ARRAY.meta(),
    SAME_SIZE.meta(),
    VOLUME.meta(),
    COMPOUND_ELEMENTS.meta(),
    INVALID_BLOCK.meta(),
    NOT_A_NUMBER.meta(),
    COMPOUND_VALUE.meta(),
    COMPOUND_FIELD.meta(),
    LIST_FIELD.meta(),
    COMPOUND_AT.meta(),
    INVALID_ENTITY.meta(),
    NOT_NAMESPACED.meta(),
];
