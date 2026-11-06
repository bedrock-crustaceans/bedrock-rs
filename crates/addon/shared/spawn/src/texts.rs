//! Vanilla's content-log texts of the spawn loaders, as the folders quote them. `{}` is where
//! `VanillaText::Formatted` puts each argument (the identifier); fmt's `{{` / `}}` escapes are
//! written as the single braces the game prints.

/// The object-form text of one condition key: `minecraft:<key>` is part of the template.
macro_rules! object_form {
    ($key:literal) => {
        concat!(
            "Spawn rules for minecraft:",
            $key,
            " in {} need to be put inside a json object - it has to start with an \"{\" and end with a \"}\" - using default values"
        )
    };
}

/// `minecraft:weight` is not an object.
pub const WEIGHT_OBJECT: &str = object_form!("weight");
/// `minecraft:density_limit` is not an object.
pub const DENSITY_OBJECT: &str = object_form!("density_limit");
/// `minecraft:brightness_filter` is not an object.
pub const BRIGHTNESS_OBJECT: &str = object_form!("brightness_filter");
/// `minecraft:height_filter` is not an object.
pub const HEIGHT_OBJECT: &str = object_form!("height_filter");
/// `minecraft:distance_filter` is not an object.
pub const DISTANCE_OBJECT: &str = object_form!("distance_filter");
/// `minecraft:delay_filter` is not an object.
pub const DELAY_OBJECT: &str = object_form!("delay_filter");
/// `minecraft:mob_event_filter` is not an object.
pub const MOB_EVENT_OBJECT: &str = object_form!("mob_event_filter");

// A literal copy of the text: no identifier, single braces.
/// `minecraft:world_age_filter` is not an object.
pub const WORLD_AGE_OBJECT: &str = "Spawn rules for minecraft:world_age_filter need to be put inside a json object - it has to start with an \"{\" and end with a \"}\" - using default values";

// The height check names `distance_filter` in vanilla's text.
/// `height_filter.min > max`.
pub const HEIGHT_SWAPPED: &str =
    "Spawn rules for minecraft:distance_filter specified a min height > max.";
/// `distance_filter.min > max`.
pub const DISTANCE_SWAPPED: &str =
    "Spawn rules for minecraft:distance_filter specified a min distance > max.";
/// `world_age_filter.min > max`.
pub const WORLD_AGE_SWAPPED: &str =
    "Spawn rules for minecraft:world_age_filter specified a min world age > max.";
/// `delay_filter.min > max`.
pub const DELAY_SWAPPED: &str = "Spawn rules for {} specified a min delay > max.";
/// `delay_filter.identifier` empty or missing.
pub const DELAY_WITHOUT_IDENTIFIER: &str =
    "Spawn rules for {} specified a delay without a target identifier.";
/// `player_in_village_filter.distance` missing or `null`.
pub const VILLAGE_DISTANCE_MISSING: &str =
    "Missing \"distance\" parameter for minecraft:player_in_village_filter";

// The literal ends in a newline, which is trimmed.
/// `description.identifier` missing, `null` or not a string.
pub const MISSING_IDENTIFIER: &str = "Missing identifier in spawn rule file";
/// The root is not a JSON object.
pub const NOT_AN_OBJECT: &str = "Spawn rules json was not formatted as an object";

/// Every template with arguments, for the arity test.
#[doc(hidden)]
pub const FORMATTED: &[&str] = &[
    WEIGHT_OBJECT,
    DENSITY_OBJECT,
    BRIGHTNESS_OBJECT,
    HEIGHT_OBJECT,
    DISTANCE_OBJECT,
    DELAY_OBJECT,
    MOB_EVENT_OBJECT,
    DELAY_SWAPPED,
    DELAY_WITHOUT_IDENTIFIER,
];
