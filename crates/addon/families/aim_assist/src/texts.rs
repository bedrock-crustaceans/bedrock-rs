//! Vanilla's content-log texts of the aim-assist documents (vanilla's spelling kept). The cereal texts are in `camera::shape`.

// Texts of the preset loader.
/// A1: a preset `identifier` without a namespace.
pub(crate) const PRESET_NAMESPACE: &str = "Namespace required for preset Id: {}";
/// A2: a preset `identifier` that is not `namespace:name`.
pub(crate) const PRESET_FORMAT: &str =
    "Failed to add aim assist preset with invalid namespace Id: {}";
/// A3: a category reference without a namespace.
pub(crate) const CATEGORY_ID_NAMESPACE: &str = "Namespace required for category Id: {}";
/// A4: a category reference that is not `namespace:name`.
pub(crate) const CATEGORY_ID_FORMAT: &str = "Invalid namespace format for category Id: {}";
// Texts of the categories loader.
/// A5: a category `name` without a namespace.
pub(crate) const CATEGORY_NAMESPACE: &str = "Namespace required for category: {}";
/// A6: a category `name` that is not `namespace:name`.
pub(crate) const CATEGORY_FORMAT: &str = "Invalid namespace format for category: {}";
// Text of the cereal constraint.
/// A negative priority.
pub(crate) const INVALID_NUMERIC: &str =
    bedrock_addon_core::diagnostic::cereal::INVALID_NUMERIC_VALUE_TEXT;

/// Every template with arguments and its number of arguments, for the arity test.
#[cfg(test)]
pub(crate) const FORMATTED: &[(&str, usize)] = &[
    (PRESET_NAMESPACE, 1),
    (PRESET_FORMAT, 1),
    (CATEGORY_ID_NAMESPACE, 1),
    (CATEGORY_ID_FORMAT, 1),
    (CATEGORY_NAMESPACE, 1),
    (CATEGORY_FORMAT, 1),
];
