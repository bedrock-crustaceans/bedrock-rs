//! Vanilla's content-log texts of the structure loader: each constructor fills the arguments
//! of one definition of [`codes`](super::codes).

use super::codes;
use bedrock_addon_core::diagnostic::Code;

/// `"%s" field, a required field, is missing from the structure.`
pub(super) fn required(field: &str) -> Code {
    codes::REQUIRED.with([field.to_owned()])
}

/// `"%s" field has a value larger than the current version.  We expect a format version with
/// a value at most %d and got %d.` (two spaces after "version.").
pub(super) fn larger_version(field: &str, current: i32, value: i32) -> Code {
    codes::LARGER_VERSION.with([field.to_owned(), current.to_string(), value.to_string()])
}

/// `"%s" field has a value smaller than the current version.  We expect a format version with
/// a value at most %d and got %d.`
pub(super) fn smaller_version(field: &str, current: i32, value: i32) -> Code {
    codes::SMALLER_VERSION.with([field.to_owned(), current.to_string(), value.to_string()])
}

/// `"%s" field is not an integer.`
pub(super) fn not_an_integer(field: &str) -> Code {
    codes::NOT_AN_INTEGER.with([field.to_owned()])
}

/// `"%s" field has an invalid value of %i. Valid values are %i and up to %i.`
pub(super) fn invalid_value(field: &str, value: i32, min: i32, max: i32) -> Code {
    codes::INVALID_VALUE.with([
        field.to_owned(),
        value.to_string(),
        min.to_string(),
        max.to_string(),
    ])
}

/// `The "%s" field expects 3 elements.`
pub(super) fn three_elements(field: &str) -> Code {
    codes::THREE_ELEMENTS.with([field.to_owned()])
}

/// `The "%s" field shouldn't have any negative values.`
pub(super) fn negative(field: &str) -> Code {
    codes::NEGATIVE.with([field.to_owned()])
}

/// `The "%s" field should be an array with 2 arrays and instead we have %d arrays.`
pub(super) fn two_arrays(field: &str, count: usize) -> Code {
    codes::TWO_ARRAYS.with([field.to_owned(), count.to_string()])
}

/// `The "%s" field's first array is either missing or not a list.`
pub(super) fn first_array(field: &str) -> Code {
    codes::FIRST_ARRAY.with([field.to_owned()])
}

/// `The "%s" field's arrays need to both be the same size.`
pub(super) fn same_size(field: &str) -> Code {
    codes::SAME_SIZE.with([field.to_owned()])
}

/// `The "%s" field should have as many elements as defined by the "%s" field.`
pub(super) fn volume(field: &str, size: &str) -> Code {
    codes::VOLUME.with([field.to_owned(), size.to_owned()])
}

/// `Expected a compound tag when parsing elements of "%s" field.`
pub(super) fn compound_elements(field: &str) -> Code {
    codes::COMPOUND_ELEMENTS.with([field.to_owned()])
}

/// `At index %d while reading "%s", we found an invalid block in the palette, we were
/// expecting a compound tag here.` (the field is `block_palette`)
pub(super) fn invalid_block(index: usize) -> Code {
    codes::INVALID_BLOCK.with([index.to_string(), "block_palette".to_owned()])
}

/// `Expected a string formatted as a number when parsing contents of the "%s" field, but got
/// "%s"` (no final period)
pub(super) fn not_a_number(field: &str, key: &str) -> Code {
    codes::NOT_A_NUMBER.with([field.to_owned(), key.to_owned()])
}

/// `Expected a compound tag when parsing elements of "%s" field at the value for "%s".`
pub(super) fn compound_value(field: &str, key: &str) -> Code {
    codes::COMPOUND_VALUE.with([field.to_owned(), key.to_owned()])
}

/// `Expected a compound tag for the "%s" field.`
pub(super) fn compound_field(field: &str) -> Code {
    codes::COMPOUND_FIELD.with([field.to_owned()])
}

/// `Expected a list tag for the "%s" field.`
pub(super) fn list_field(field: &str) -> Code {
    codes::LIST_FIELD.with([field.to_owned()])
}

/// `Expected a compound tag at index %d when parsing elements of "%s" field.`
pub(super) fn compound_at(index: usize, field: &str) -> Code {
    codes::COMPOUND_AT.with([index.to_string(), field.to_owned()])
}

/// `At index %d while reading "%s", we found an invalid entity in the list of entities.`
/// (the field is `entities`)
pub(super) fn invalid_entity(index: usize) -> Code {
    codes::INVALID_ENTITY.with([index.to_string(), "entities".to_owned()])
}

/// `Structure '%s' must be prefixed by a namespace`, written when a structure is looked up.
pub(super) fn not_namespaced(id: &str) -> Code {
    codes::NOT_NAMESPACED.with([id.to_owned()])
}
