//! The content-log texts of the feature loader stage.

/// The client's text for a file that is no valid JSON.
pub(crate) const INVALID_JSON: &str = "[Feature Registry] Passed in contents could not be evaluated as valid JSON in the following file: {}";
/// The client's text for a file without a description identifier.
pub(crate) const MISSING_IDENTIFIER: &str =
    "[Feature Registry] Missing description identifier for JSON in the following file: {}";
/// The client's text for an empty description identifier.
pub(crate) const EMPTY_IDENTIFIER: &str =
    "[Feature Registry] Empty description identifier for JSON in the following file: {}";
/// The client's text for an identifier that is not `namespace:id`.
pub(crate) const NOT_NAMESPACED: &str =
    "The feature identifier '{}' should be in the format namespace:id";
/// The built-in definitions' namespace rule.
pub(crate) const NOT_MINECRAFT: &str =
    "The feature identifer '{}' should be in the 'minecraft' namespace";
/// The client's text for an internal feature type used outside the game's packs.
pub(crate) const INTERNAL_TYPE: &str = "The feature uses the feature type '{}', which is considered internal and thus not compatible with this engine version of the game.";
/// The client's text for a second feature of one identifier in a pack (engine 1.26.40 and
/// later).
pub(crate) const DUPLICATE: &str = "Feature with identifier '{}' already exists";
