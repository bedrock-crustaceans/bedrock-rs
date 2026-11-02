//! Vanilla's content-log texts of the structure loader. Each constructor fills one template; the
//! unit test below checks that every template gets one argument per `{}`.

use bedrock_addon_core::diagnostic::VanillaText;

/// A template with its arguments.
fn formatted(template: &'static str, args: Vec<String>) -> VanillaText {
    VanillaText::Formatted { template, args }
}

/// `"%s" field, a required field, is missing from the structure.`
pub(super) fn required(field: &str) -> VanillaText {
    formatted(
        "\"{}\" field, a required field, is missing from the structure.",
        vec![field.to_owned()],
    )
}

/// `"%s" field has a value larger than the current version.  We expect a format version with
/// a value at most %d and got %d.` (two spaces after "version.").
pub(super) fn larger_version(field: &str, current: i32, value: i32) -> VanillaText {
    formatted(
        "\"{}\" field has a value larger than the current version.  We expect a format version with a value at most {} and got {}.",
        vec![field.to_owned(), current.to_string(), value.to_string()],
    )
}

/// `"%s" field has a value smaller than the current version.  We expect a format version with
/// a value at most %d and got %d.`
pub(super) fn smaller_version(field: &str, current: i32, value: i32) -> VanillaText {
    formatted(
        "\"{}\" field has a value smaller than the current version.  We expect a format version with a value at most {} and got {}.",
        vec![field.to_owned(), current.to_string(), value.to_string()],
    )
}

/// `"%s" field is not an integer.`
pub(super) fn not_an_integer(field: &str) -> VanillaText {
    formatted("\"{}\" field is not an integer.", vec![field.to_owned()])
}

/// `"%s" field has an invalid value of %i. Valid values are %i and up to %i.`
pub(super) fn invalid_value(field: &str, value: i32, min: i32, max: i32) -> VanillaText {
    formatted(
        "\"{}\" field has an invalid value of {}. Valid values are {} and up to {}.",
        vec![
            field.to_owned(),
            value.to_string(),
            min.to_string(),
            max.to_string(),
        ],
    )
}

/// `The "%s" field expects 3 elements.`
pub(super) fn three_elements(field: &str) -> VanillaText {
    formatted(
        "The \"{}\" field expects 3 elements.",
        vec![field.to_owned()],
    )
}

/// `The "%s" field shouldn't have any negative values.`
pub(super) fn negative(field: &str) -> VanillaText {
    formatted(
        "The \"{}\" field shouldn't have any negative values.",
        vec![field.to_owned()],
    )
}

/// `The "%s" field should be an array with 2 arrays and instead we have %d arrays.`
pub(super) fn two_arrays(field: &str, count: usize) -> VanillaText {
    formatted(
        "The \"{}\" field should be an array with 2 arrays and instead we have {} arrays.",
        vec![field.to_owned(), count.to_string()],
    )
}

/// `The "%s" field's first array is either missing or not a list.`
pub(super) fn first_array(field: &str) -> VanillaText {
    formatted(
        "The \"{}\" field's first array is either missing or not a list.",
        vec![field.to_owned()],
    )
}

/// `The "%s" field's arrays need to both be the same size.`
pub(super) fn same_size(field: &str) -> VanillaText {
    formatted(
        "The \"{}\" field's arrays need to both be the same size.",
        vec![field.to_owned()],
    )
}

/// `The "%s" field should have as many elements as defined by the "%s" field.`
pub(super) fn volume(field: &str, size: &str) -> VanillaText {
    formatted(
        "The \"{}\" field should have as many elements as defined by the \"{}\" field.",
        vec![field.to_owned(), size.to_owned()],
    )
}

/// `Expected a compound tag when parsing elements of "%s" field.`
pub(super) fn compound_elements(field: &str) -> VanillaText {
    formatted(
        "Expected a compound tag when parsing elements of \"{}\" field.",
        vec![field.to_owned()],
    )
}

/// `At index %d while reading "%s", we found an invalid block in the palette, we were
/// expecting a compound tag here.` (the field is `block_palette`)
pub(super) fn invalid_block(index: usize) -> VanillaText {
    formatted(
        "At index {} while reading \"{}\", we found an invalid block in the palette, we were expecting a compound tag here.",
        vec![index.to_string(), "block_palette".to_owned()],
    )
}

/// `Expected a string formatted as a number when parsing contents of the "%s" field, but got
/// "%s"` (no final period)
pub(super) fn not_a_number(field: &str, key: &str) -> VanillaText {
    formatted(
        "Expected a string formatted as a number when parsing contents of the \"{}\" field, but got \"{}\"",
        vec![field.to_owned(), key.to_owned()],
    )
}

/// `Expected a compound tag when parsing elements of "%s" field at the value for "%s".`
pub(super) fn compound_value(field: &str, key: &str) -> VanillaText {
    formatted(
        "Expected a compound tag when parsing elements of \"{}\" field at the value for \"{}\".",
        vec![field.to_owned(), key.to_owned()],
    )
}

/// `Expected a compound tag for the "%s" field.`
pub(super) fn compound_field(field: &str) -> VanillaText {
    formatted(
        "Expected a compound tag for the \"{}\" field.",
        vec![field.to_owned()],
    )
}

/// `Expected a list tag for the "%s" field.`
pub(super) fn list_field(field: &str) -> VanillaText {
    formatted(
        "Expected a list tag for the \"{}\" field.",
        vec![field.to_owned()],
    )
}

/// `Expected a compound tag at index %d when parsing elements of "%s" field.`
pub(super) fn compound_at(index: usize, field: &str) -> VanillaText {
    formatted(
        "Expected a compound tag at index {} when parsing elements of \"{}\" field.",
        vec![index.to_string(), field.to_owned()],
    )
}

/// `At index %d while reading "%s", we found an invalid entity in the list of entities.`
/// (the field is `entities`)
pub(super) fn invalid_entity(index: usize) -> VanillaText {
    formatted(
        "At index {} while reading \"{}\", we found an invalid entity in the list of entities.",
        vec![index.to_string(), "entities".to_owned()],
    )
}

/// `Structure '%s' must be prefixed by a namespace`, written when a structure is looked up.
pub(super) fn not_namespaced(id: &str) -> VanillaText {
    formatted(
        "Structure '{}' must be prefixed by a namespace",
        vec![id.to_owned()],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The template and argument count of a formatted text.
    fn arity(text: &VanillaText) -> (usize, usize) {
        match text {
            VanillaText::Formatted { template, args } => {
                (template.matches("{}").count(), args.len())
            }
            other => panic!("not formatted: {other:?}"),
        }
    }

    #[test]
    fn every_template_gets_one_argument_per_placeholder() {
        let texts = [
            required("x"),
            larger_version("x", 1, 2),
            smaller_version("x", 1, -1),
            not_an_integer("x"),
            invalid_value("x", 2, 0, 1),
            three_elements("x"),
            negative("x"),
            two_arrays("x", 1),
            first_array("x"),
            same_size("x"),
            volume("x", "y"),
            compound_elements("x"),
            invalid_block(0),
            not_a_number("x", "y"),
            compound_value("x", "y"),
            compound_field("x"),
            list_field("x"),
            compound_at(0, "x"),
            invalid_entity(0),
            not_namespaced("x"),
        ];
        for text in &texts {
            let (placeholders, args) = arity(text);
            assert_eq!(placeholders, args, "{text:?}");
        }
    }
}
