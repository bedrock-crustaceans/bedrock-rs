//! The game's generic validation texts: the texts of cereal's schemas and constraints, of the
//! Minecraft Document envelope and of the game's older schema tree, each with the code the crate
//! files the condition under.
//!
//! A family that meets one of these conditions takes its finding from here, so that every
//! family prints the same text the game prints: [`cereal::enum_expects`](enum_expects) for an
//! enum string outside its set, [`NumericRange::code`] for a number outside its range, and so on.
//! The severity and the path stay the family's. Every text here is an error in both of the
//! game's log masks except the unknown member and the unknown component, which only the strict
//! mask makes one. Each constructor documents the text it produces and the condition it reports.

use super::{DiagnosticCode, VanillaText};

/// `template` with `args`.
#[allow(
    clippy::unnecessary_wraps,
    reason = "the `text` member of every code is an `Option`"
)]
fn formatted(template: &'static str, args: Vec<String>) -> Option<VanillaText> {
    Some(VanillaText::Formatted { template, args })
}

/// A fixed text.
#[allow(
    clippy::unnecessary_wraps,
    reason = "the `text` member of every code is an `Option`"
)]
const fn fixed(text: &'static str) -> Option<VanillaText> {
    Some(VanillaText::Fixed(text))
}

/// A number as cereal writes it into a message: without a fraction as an integer, otherwise
/// as C++'s `std::to_string(double)` does, with `%f` (six decimals). Every numeric constraint
/// converts its bound to `double` first.
#[must_use]
pub fn number(value: f64) -> String {
    if value.fract() == 0.0 && value.is_finite() {
        // The game's float-to-integer conversion truncates towards zero and saturates, as the
        // cast does.
        #[allow(
            clippy::cast_possible_truncation,
            reason = "the game's own conversion, saturating"
        )]
        let whole = value as i64;
        whole.to_string()
    } else {
        format!("{value:.6}")
    }
}

// ---- composite (object) loading -----------------------------------------------------------

/// The text of [`member_not_in_schema`], for a family that files it under another code or
/// prints it as a plain line.
pub const MEMBER_NOT_IN_SCHEMA_TEXT: &str =
    "this member was found in the input, but is not present in the Schema";

/// `this member was found in the input, but is not present in the Schema`: an unknown member of
/// an object.
#[must_use]
pub const fn member_not_in_schema() -> DiagnosticCode {
    DiagnosticCode::UnrecognizedField {
        text: fixed(MEMBER_NOT_IN_SCHEMA_TEXT),
    }
}

/// `this component was found in the input, but is not present in the Schema`: an unknown
/// component.
#[must_use]
pub const fn component_not_in_schema() -> DiagnosticCode {
    DiagnosticCode::UnrecognizedComponent {
        text: fixed("this component was found in the input, but is not present in the Schema"),
    }
}

/// The text of [`missing_required_field`].
pub const MISSING_REQUIRED_FIELD_TEXT: &str = "missing required field";

/// `missing required field`: an object lacks a member its schema requires.
#[must_use]
pub const fn missing_required_field() -> DiagnosticCode {
    DiagnosticCode::MissingRequiredField {
        text: fixed(MISSING_REQUIRED_FIELD_TEXT),
    }
}

/// `expected an object`: something else stands where an object is required.
#[must_use]
pub const fn expected_object() -> DiagnosticCode {
    DiagnosticCode::InvalidType {
        text: fixed("expected an object"),
    }
}

/// `too few input components, expected at least <min>`: a component storage holds fewer
/// components than its schema demands.
#[must_use]
pub fn too_few_components(min: usize) -> DiagnosticCode {
    DiagnosticCode::InvalidValue {
        text: formatted(
            "too few input components, expected at least {}",
            vec![min.to_string()],
        ),
    }
}

// ---- enums --------------------------------------------------------------------------------

/// `invalid enum value`: the line the game logs after [`enum_expects`] or
/// [`enum_expects_string`].
#[must_use]
pub const fn invalid_enum_value() -> DiagnosticCode {
    DiagnosticCode::InvalidEnumValue {
        text: fixed("invalid enum value"),
    }
}

/// How many of an enum's names [`enum_expects`] lists before `, ... (<n> others)`.
const ENUM_NAMES_SHOWN: usize = 3;

/// An enum given a string outside its set. `bound` is the enum's cereal name; `names` its
/// names **in the order of their values** (the game sorts the enum's name and value pairs by
/// value before listing them); `found` the string as written.
///
/// `This value is bound to '<bound>' and expects one of the following strings: '<a>', '<b>',
/// '<c>'` then `, ... (<n> others)` when there are more than three names, then `. Found:
/// '<found>'`. The count in the tail is the number of names less the three shown.
#[must_use]
pub fn enum_expects(bound: &str, names: &[&str], found: &str) -> DiagnosticCode {
    let mut list = names
        .iter()
        .take(ENUM_NAMES_SHOWN)
        .map(|name| format!("'{name}'"))
        .collect::<Vec<_>>()
        .join(", ");
    if names.len() > ENUM_NAMES_SHOWN {
        list.push_str(", ... (");
        list.push_str(&(names.len() - ENUM_NAMES_SHOWN).to_string());
        list.push_str(" others)");
    }
    DiagnosticCode::InvalidEnumValue {
        text: formatted(
            "This value is bound to '{}' and expects one of the following strings: {}. Found: '{}'",
            vec![bound.to_owned(), list, found.to_owned()],
        ),
    }
}

/// `This value is bound to '<bound>' and expects a "named value" (i.e. a string)`: an enum given
/// something other than a string.
#[must_use]
pub fn enum_expects_string(bound: &str) -> DiagnosticCode {
    DiagnosticCode::InvalidType {
        text: formatted(
            "This value is bound to '{}' and expects a \"named value\" (i.e. a string)",
            vec![bound.to_owned()],
        ),
    }
}

// ---- scalars, ranges, sizes ---------------------------------------------------------------

/// The text of [`invalid_numeric_value`].
pub const INVALID_NUMERIC_VALUE_TEXT: &str = "invalid numeric value";

/// `invalid numeric value`: no number (or boolean) where one is expected.
#[must_use]
pub const fn invalid_numeric_value() -> DiagnosticCode {
    DiagnosticCode::InvalidType {
        text: fixed(INVALID_NUMERIC_VALUE_TEXT),
    }
}

/// The bounds of a numeric constraint: each bound is absent when the constraint has none (the
/// game then uses the type's lowest or highest value), and each says whether it is exclusive.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NumericRange {
    /// The lower bound.
    pub min: Option<f64>,
    /// The upper bound.
    pub max: Option<f64>,
    /// The lower bound is excluded (`> min`).
    pub exclusive_min: bool,
    /// The upper bound is excluded (`< max`).
    pub exclusive_max: bool,
}

impl NumericRange {
    /// `min <= x <= max`.
    #[must_use]
    pub const fn inclusive(min: f64, max: f64) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
            exclusive_min: false,
            exclusive_max: false,
        }
    }

    /// `x >= min`.
    #[must_use]
    pub const fn at_least(min: f64) -> Self {
        Self {
            min: Some(min),
            max: None,
            exclusive_min: false,
            exclusive_max: false,
        }
    }

    /// `x <= max`.
    #[must_use]
    pub const fn at_most(max: f64) -> Self {
        Self {
            min: None,
            max: Some(max),
            exclusive_min: false,
            exclusive_max: false,
        }
    }

    /// Whether `value` is inside.
    #[must_use]
    pub fn contains(&self, value: f64) -> bool {
        let above = match self.min {
            Some(min) if self.exclusive_min => value > min,
            Some(min) => value >= min,
            None => true,
        };
        let below = match self.max {
            Some(max) if self.exclusive_max => value < max,
            Some(max) => value <= max,
            None => true,
        };
        above && below
    }

    /// The text of a value outside, the same for every numeric type: with both bounds `the input
    /// is outside of the accepted range ` + `[` or `(` + min + `, ` + max + `]` or `)`; with one,
    /// `the input must be a number` + ` >= ` / ` > ` + min, or ` <= ` / ` < ` + max. Bounds as
    /// [`number`] writes them.
    #[must_use]
    pub fn text(&self) -> VanillaText {
        let text = match (self.min, self.max) {
            (Some(min), Some(max)) => format!(
                "the input is outside of the accepted range {}{}, {}{}",
                if self.exclusive_min { '(' } else { '[' },
                number(min),
                number(max),
                if self.exclusive_max { ')' } else { ']' },
            ),
            (None, Some(max)) => format!(
                "the input must be a number{}{}",
                if self.exclusive_max { " < " } else { " <= " },
                number(max)
            ),
            (Some(min), None) => format!(
                "the input must be a number{}{}",
                if self.exclusive_min { " > " } else { " >= " },
                number(min)
            ),
            // No bound: no constraint fails; the game's text would end after "number".
            (None, None) => "the input must be a number".to_owned(),
        };
        VanillaText::Formatted {
            template: "{}",
            args: vec![text],
        }
    }

    /// The finding's code: `InvalidValue` with [`Self::text`].
    #[must_use]
    pub fn code(&self) -> DiagnosticCode {
        DiagnosticCode::InvalidValue {
            text: Some(self.text()),
        }
    }
}

/// `<value> is not multiple of <step>`, for a number that breaks a multiple-of constraint: both
/// as [`number`] writes them.
#[must_use]
pub fn not_multiple_of(value: f64, step: f64) -> DiagnosticCode {
    DiagnosticCode::InvalidValue {
        text: formatted(
            "{} is not multiple of {}",
            vec![number(value), number(step)],
        ),
    }
}

/// The template of [`string_mismatch`] (one argument: the regex).
pub const STRING_MISMATCH_TEXT: &str = "expected string to match the \"{}\" regex";

/// `expected string to match the "<regex>" regex`: a string breaks a pattern constraint.
#[must_use]
pub fn string_mismatch(regex: &str) -> DiagnosticCode {
    DiagnosticCode::InvalidValue {
        text: formatted(STRING_MISMATCH_TEXT, vec![regex.to_owned()]),
    }
}

/// `expected array with at least <n> elements`; the text says `elements` whatever `n` is.
#[must_use]
pub fn array_too_small(min: usize) -> DiagnosticCode {
    DiagnosticCode::InvalidValue {
        text: formatted(
            "expected array with at least {} elements",
            vec![min.to_string()],
        ),
    }
}

/// `expected object with at least <n> elements`, for a map or object under a minimum-size
/// constraint.
#[must_use]
pub fn object_too_small(min: usize) -> DiagnosticCode {
    DiagnosticCode::InvalidValue {
        text: formatted(
            "expected object with at least {} elements",
            vec![min.to_string()],
        ),
    }
}

/// `No duplicates allowed`: an array with a unique-items constraint holds a repeat.
#[must_use]
pub const fn no_duplicates() -> DiagnosticCode {
    DiagnosticCode::InvalidValue {
        text: fixed("No duplicates allowed"),
    }
}

/// `key <key> is not allowed`: a component storage holds a key its constraint forbids.
#[must_use]
pub fn key_not_allowed(key: &str) -> DiagnosticCode {
    DiagnosticCode::InvalidValue {
        text: formatted("key {} is not allowed", vec![key.to_owned()]),
    }
}

/// `too many input elements, expected no more than <max>`: the size cap of a vector; the same
/// text ends the maximum size of a sequence container constraint.
#[must_use]
pub fn too_many_elements(max: usize) -> DiagnosticCode {
    DiagnosticCode::InvalidValue {
        text: formatted(
            "too many input elements, expected no more than {}",
            vec![max.to_string()],
        ),
    }
}

/// `too many characters, expected no more than <max>`: the size cap of a string.
#[must_use]
pub fn too_many_characters(max: usize) -> DiagnosticCode {
    DiagnosticCode::InvalidValue {
        text: formatted(
            "too many characters, expected no more than {}",
            vec![max.to_string()],
        ),
    }
}

/// `too many input elements for an associative container`: the size cap of a map.
#[must_use]
pub const fn too_many_map_elements() -> DiagnosticCode {
    DiagnosticCode::InvalidValue {
        text: fixed("too many input elements for an associative container"),
    }
}

// ---- the Minecraft Document envelope ------------------------------------------------------

/// `The payload of the Minecraft Document is missing`.
#[must_use]
pub const fn payload_missing() -> DiagnosticCode {
    DiagnosticCode::MissingPayload {
        text: fixed("The payload of the Minecraft Document is missing"),
    }
}

/// `The payload key does not match the expected one. Expected: '<expected>'. Actual:
/// '<actual>'.` The crate files it as the missing payload it is to the family.
#[must_use]
pub fn payload_key_mismatch(expected: &str, actual: &str) -> DiagnosticCode {
    DiagnosticCode::MissingPayload {
        text: formatted(
            "The payload key does not match the expected one. Expected: '{}'. Actual: '{}'.",
            vec![expected.to_owned(), actual.to_owned()],
        ),
    }
}

// ---- the older schema tree ----------------------------------------------------------------

/// The template of [`required_child`] (one argument: the name).
pub const REQUIRED_CHILD_TEXT: &str = "Required child {} not found";

/// `Required child <name> not found`, logged at level 3.
#[must_use]
pub fn required_child(name: &str) -> DiagnosticCode {
    DiagnosticCode::MissingRequiredField {
        text: formatted(REQUIRED_CHILD_TEXT, vec![name.to_owned()]),
    }
}

/// `Node has too few children (<actual> < <min>)`, logged at level 3.
#[must_use]
pub fn too_few_children(actual: usize, min: usize) -> DiagnosticCode {
    DiagnosticCode::InvalidValue {
        text: formatted(
            "Node has too few children ({} < {})",
            vec![actual.to_string(), min.to_string()],
        ),
    }
}

/// `Node has too many children (<actual> > <max>)`, logged at level 3.
#[must_use]
pub fn too_many_children(actual: usize, max: usize) -> DiagnosticCode {
    DiagnosticCode::InvalidValue {
        text: formatted(
            "Node has too many children ({} > {})",
            vec![actual.to_string(), max.to_string()],
        ),
    }
}

/// The template of [`unknown_child_option`] (one argument: the allowed types).
pub const UNKNOWN_CHILD_OPTION_TEXT: &str = "unknown child schema option type.  Allowed types: {}";

/// `unknown child schema option type.  Allowed types: <types>`, logged at level 3. The text has
/// two spaces before `Allowed`.
#[must_use]
pub fn unknown_child_option(allowed: &str) -> DiagnosticCode {
    DiagnosticCode::InvalidType {
        text: formatted(UNKNOWN_CHILD_OPTION_TEXT, vec![allowed.to_owned()]),
    }
}

/// `Version <version> below min required version (<min>)`: a version below the node's minimum.
#[must_use]
pub fn version_below_min(version: &str, min: &str) -> DiagnosticCode {
    DiagnosticCode::UnsupportedVersion {
        text: formatted(
            "Version {} below min required version ({})",
            vec![version.to_owned(), min.to_owned()],
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The code's name and its rendered text.
    fn shown(code: &DiagnosticCode) -> (&'static str, String) {
        (code.name(), code.to_string())
    }

    #[test]
    fn e2_lists_three_names_then_the_count_of_the_others_then_the_value() {
        let facing = ["down", "up", "north", "south", "west", "east"];
        assert_eq!(
            shown(&enum_expects("Facing", &facing, "sideways")),
            (
                "InvalidEnumValue",
                "This value is bound to 'Facing' and expects one of the following strings: 'down', 'up', 'north', ... (3 others). Found: 'sideways'".to_owned()
            )
        );
        assert_eq!(
            enum_expects("Rule Condition", &["a", "b", "c"], "x").to_string(),
            "This value is bound to 'Rule Condition' and expects one of the following strings: 'a', 'b', 'c'. Found: 'x'"
        );
        assert_eq!(
            enum_expects("One", &["a"], "x").to_string(),
            "This value is bound to 'One' and expects one of the following strings: 'a'. Found: 'x'"
        );
        assert_eq!(
            shown(&enum_expects_string("Facing")),
            (
                "InvalidType",
                "This value is bound to 'Facing' and expects a \"named value\" (i.e. a string)"
                    .to_owned()
            )
        );
        assert_eq!(
            shown(&invalid_enum_value()),
            ("InvalidEnumValue", "invalid enum value".to_owned())
        );
    }

    #[test]
    fn f2_writes_range_texts_and_numbers_as_cereal_does() {
        assert_eq!(number(0.0), "0");
        assert_eq!(number(15.0), "15");
        assert_eq!(number(-8.0), "-8");
        assert_eq!(number(0.9), "0.900000");
        assert_eq!(number(0.25), "0.250000");
        assert_eq!(
            shown(&NumericRange::inclusive(0.0, 0.9).code()),
            (
                "InvalidValue",
                "the input is outside of the accepted range [0, 0.900000]".to_owned()
            )
        );
        let open = NumericRange {
            exclusive_min: true,
            exclusive_max: true,
            ..NumericRange::inclusive(0.0, 100.0)
        };
        assert_eq!(
            open.text().to_string(),
            "the input is outside of the accepted range (0, 100)"
        );
        assert_eq!(
            NumericRange::at_most(15.0).text().to_string(),
            "the input must be a number <= 15"
        );
        assert_eq!(
            NumericRange::at_least(0.0).text().to_string(),
            "the input must be a number >= 0"
        );
        let above = NumericRange {
            exclusive_min: true,
            ..NumericRange::at_least(0.0)
        };
        assert_eq!(above.text().to_string(), "the input must be a number > 0");
        let below = NumericRange {
            exclusive_max: true,
            ..NumericRange::at_most(1.0)
        };
        assert_eq!(below.text().to_string(), "the input must be a number < 1");
        assert!(NumericRange::inclusive(0.0, 0.9).contains(0.9));
        assert!(!NumericRange::inclusive(0.0, 0.9).contains(0.91));
        assert!(!above.contains(0.0));
        assert_eq!(
            shown(&not_multiple_of(45.0, 90.0)),
            ("InvalidValue", "45 is not multiple of 90".to_owned())
        );
    }

    #[test]
    fn f3_to_f8_and_d8_d11_name_their_bound() {
        let cases = [
            (
                string_mismatch("^\\S+$"),
                "InvalidValue",
                "expected string to match the \"^\\S+$\" regex",
            ),
            (
                array_too_small(1),
                "InvalidValue",
                "expected array with at least 1 elements",
            ),
            (
                object_too_small(2),
                "InvalidValue",
                "expected object with at least 2 elements",
            ),
            (no_duplicates(), "InvalidValue", "No duplicates allowed"),
            (
                key_not_allowed("side"),
                "InvalidValue",
                "key side is not allowed",
            ),
            (
                too_many_elements(64),
                "InvalidValue",
                "too many input elements, expected no more than 64",
            ),
            (
                too_many_characters(64),
                "InvalidValue",
                "too many characters, expected no more than 64",
            ),
            (
                too_many_map_elements(),
                "InvalidValue",
                "too many input elements for an associative container",
            ),
            (
                too_few_components(1),
                "InvalidValue",
                "too few input components, expected at least 1",
            ),
        ];
        for (code, name, text) in cases {
            assert_eq!(shown(&code), (name, text.to_owned()));
        }
    }

    #[test]
    fn d_f_g_fixed_texts_and_codes() {
        let cases = [
            (
                member_not_in_schema(),
                "UnrecognizedField",
                "this member was found in the input, but is not present in the Schema",
            ),
            (
                component_not_in_schema(),
                "UnrecognizedComponent",
                "this component was found in the input, but is not present in the Schema",
            ),
            (
                missing_required_field(),
                "MissingRequiredField",
                "missing required field",
            ),
            (expected_object(), "InvalidType", "expected an object"),
            (
                invalid_numeric_value(),
                "InvalidType",
                "invalid numeric value",
            ),
        ];
        for (code, name, text) in cases {
            assert_eq!(shown(&code), (name, text.to_owned()));
        }
    }

    #[test]
    fn a10_a11_and_the_schema_tree_texts() {
        let cases = [
            (
                payload_missing(),
                "MissingPayload",
                "The payload of the Minecraft Document is missing",
            ),
            (
                payload_key_mismatch("minecraft:block", "minecraft:item"),
                "MissingPayload",
                "The payload key does not match the expected one. Expected: 'minecraft:block'. Actual: 'minecraft:item'.",
            ),
            (
                required_child("format_version"),
                "MissingRequiredField",
                "Required child format_version not found",
            ),
            (
                too_few_children(0, 1),
                "InvalidValue",
                "Node has too few children (0 < 1)",
            ),
            (
                too_many_children(3, 2),
                "InvalidValue",
                "Node has too many children (3 > 2)",
            ),
            (
                unknown_child_option("object"),
                "InvalidType",
                "unknown child schema option type.  Allowed types: object",
            ),
            (
                version_below_min("1.8.0", "1.10.0"),
                "UnsupportedVersion",
                "Version 1.8.0 below min required version (1.10.0)",
            ),
        ];
        for (code, name, text) in cases {
            assert_eq!(shown(&code), (name, text.to_owned()));
        }
    }
}
