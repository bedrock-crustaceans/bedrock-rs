//! The recipe family's codes: the game's content-log lines for recipe files, byte for byte, each
//! a definition beside the rules that raise it. Each [`was`](CodeDef::was) the legacy code its
//! sites raised, so a pin or a filter on the code's name keeps matching.
//!
//! A line the game logs at the error level and after which the recipe loads is a kept error
//! ([`Class::KeptError`]); an ingredient's own line is kept or refuses the recipe as its site
//! says.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Disposition, Kind};

// ---- the document ----

/// `JSON: %s has an error: \n %s`: the arguments are the file's path and the reader's message,
/// which the finding carries again as its `DiagnosticDetail::Syntax`.
pub static JSON_ERROR: CodeDef<2> = CodeDef::new(
    "recipe/json_error",
    Kind::Syntax,
    Class::Error,
    "the file {} is not JSON: {}",
)
.game("JSON: {} has an error: \n {}")
.was("MalformedInput");

/// `JSON: %s root value is not a object`, of the file's path.
pub static ROOT_NOT_OBJECT: CodeDef<1> = CodeDef::new(
    "recipe/root_not_object",
    Kind::Syntax,
    Class::Error,
    "the root of {} is not an object",
)
.game("JSON: {} root value is not a object")
.was("MalformedInput");

/// `JSON: %s has no identifier`, of the file's path: no payload, or one without a non-empty
/// identifier.
pub static NO_IDENTIFIER: CodeDef<1> = CodeDef::new(
    "recipe/no_identifier",
    Kind::MissingRequired,
    Class::Error,
    "{} has no identifier",
)
.game("JSON: {} has no identifier")
.was("MissingIdentifier");

/// `JSON: %s has duplicate identifier in same pack: %s`: the file's path and the identifier.
pub static DUPLICATE_IDENTIFIER: CodeDef<2> = CodeDef::new(
    "recipe/duplicate_identifier",
    Kind::Duplicate,
    Class::Error,
    "{} repeats the identifier {} of its pack",
)
.game("JSON: {} has duplicate identifier in same pack: {}")
.was("DuplicateIdentifier");

/// No readable `format_version`: the recipe is read as 1.0.0 and loads.
pub static MISSING_VERSION_TAG: CodeDef<0> = CodeDef::new(
    "recipe/missing_version_tag",
    Kind::FormatVersion,
    Class::KeptError,
    "no readable format_version; read as 1.0.0",
)
.game("Missing version tag")
.was("MissingFormatVersion");

// ---- tags ----

/// No tags: one recipe is registered per tag, so nothing is.
pub static NO_TAGS: CodeDef<0> = CodeDef::new(
    "recipe/no_tags",
    Kind::NeverApplies,
    Class::KeptError,
    "no tags: nothing is registered",
)
.game("No tags specified")
.was("NoEffect");

/// The `smithing_table` tag on a recipe that is not a smithing one.
pub static SMITHING_TABLE_TAG_MISUSED: CodeDef<0> = CodeDef::new(
    "recipe/smithing_table_tag_misused",
    Kind::InvalidValue,
    Class::KeptError,
    "smithing_table tag on a recipe that is not a smithing one",
)
.game("smithing_table tag is only supported by recipes of type recipe_smithing_transform and of type recipe_smithing_trim")
.was("InvalidValue");

// ---- ingredients ----

/// An object `data`: the argument is the JSON value type number (7); the game goes on with an
/// empty expression, which evaluates to 0.
pub static DATA_NOT_MOLANG: CodeDef<1> = CodeDef::new(
    "recipe/data_not_molang",
    Kind::WrongType,
    Class::KeptError,
    "ingredient data of JSON type {} is no Molang expression",
)
.game("Type not convertible to Molang expression: {}")
.was("InvalidType");

/// A `count` on an ingredient that takes none.
pub static COUNT_NOT_ALLOWED: CodeDef<0> = CodeDef::new(
    "recipe/count_not_allowed",
    Kind::InvalidValue,
    Class::Error,
    "an item count where none is allowed",
)
.game("Recipe ingredient is invalid: Cannot specify an item count for input items")
.was("InvalidValue");

/// An ingredient without a string `item` where no tag is allowed.
pub static NO_ITEM_NAME: CodeDef<0> = CodeDef::new(
    "recipe/no_item_name",
    Kind::InvalidValue,
    Class::Error,
    "an ingredient without an item name",
)
.game("Recipe ingredient is invalid: Must contain a valid item name")
.was("InvalidValue");

/// An ingredient with neither a string `item` nor a string `tag`.
pub static NO_ITEM_OR_TAG: CodeDef<0> = CodeDef::new(
    "recipe/no_item_or_tag",
    Kind::InvalidValue,
    Class::Error,
    "an ingredient without an item name or a tag",
)
.game("Recipe ingredient is invalid: Must contain either a valid item name or item tag")
.was("InvalidValue");

/// An absent ingredient: the item lookup with an empty name fails (the argument is empty).
pub static ITEM_MISSING: CodeDef<1> = CodeDef::new(
    "recipe/item_missing",
    Kind::MissingRequired,
    Class::Error,
    "an ingredient is missing",
)
.game("The Item: {} is missing or invalid, can't make the recipe")
.was("MissingRequiredField");

/// An ingredient that is neither a string nor an object: the item lookup with an empty name
/// fails (the argument is empty).
pub static ITEM_INVALID: CodeDef<1> = CodeDef::new(
    "recipe/item_invalid",
    Kind::WrongType,
    Class::Error,
    "an ingredient is neither a string nor an object",
)
.game("The Item: {} is missing or invalid, can't make the recipe")
.was("InvalidType");

/// The item-name parser: an aux value that is no number.
pub static INVALID_AUX: CodeDef<1> = CodeDef::new(
    "recipe/invalid_aux",
    Kind::InvalidValue,
    Class::KeptError,
    "invalid aux value in item {}",
)
.game("Invalid aux value in item ID \"{}\"")
.was("InvalidValue");

/// The item-name parser: an empty namespace.
pub static EMPTY_NAMESPACE: CodeDef<1> = CodeDef::new(
    "recipe/empty_namespace",
    Kind::InvalidValue,
    Class::KeptError,
    "empty namespace in item {}",
)
.game("Empty namespace in item ID \"{}\"")
.was("InvalidValue");

/// The item-name parser: more tokens than an item name has.
pub static EXCESSIVE_TOKENS: CodeDef<1> = CodeDef::new(
    "recipe/excessive_tokens",
    Kind::InvalidValue,
    Class::KeptError,
    "too many tokens in item {}",
)
.game("Excessive number of tokens in item ID \"{}\"")
.was("InvalidValue");

// ---- results ----

/// A result that is no ingredient (also the line of a bad shapeless ingredient and of a bad
/// furnace input or output).
pub static RESULT_MALFORMED: CodeDef<0> = CodeDef::new(
    "recipe/result_malformed",
    Kind::Closing,
    Class::Error,
    "the result is malformed",
)
.game("Recipe result malformed")
.was("DocumentRejected");

/// An empty result list.
pub static NO_RESULT_ITEMS: CodeDef<0> = CodeDef::new(
    "recipe/no_result_items",
    Kind::Closing,
    Class::Error,
    "the result list is empty",
)
.game("Recipe has no result item(s)")
.was("DocumentRejected");

/// A shapeless recipe with more than one result: it loads with the first.
pub static MULTIPLE_OUTPUTS: CodeDef<0> = CodeDef::new(
    "recipe/multiple_outputs",
    Kind::UnknownMember(Disposition::Ignored),
    Class::KeptError,
    "a shapeless recipe with more than one result; the first is used",
)
.game("unsupported multiple output types for result list in shapeless")
.was("IgnoredMember");

// ---- unlock ----

/// An empty `unlock` list: the requirement is `{None, []}`.
pub static EMPTY_UNLOCK: CodeDef<1> = CodeDef::new(
    "recipe/empty_unlock",
    Kind::InvalidValue,
    Class::KeptError,
    "recipe {} has an empty unlock list",
)
.game("Recipe: {} has an empty unlocking ingredient array")
.was("InvalidValue");

/// An invalid `unlock` ingredient: the requirement is zeroed.
pub static INVALID_UNLOCK_INGREDIENT: CodeDef<1> = CodeDef::new(
    "recipe/invalid_unlock_ingredient",
    Kind::InvalidValue,
    Class::KeptError,
    "recipe {} has an invalid unlock ingredient",
)
.game("Recipe: {} has an invalid unlock ingredient")
.was("InvalidValue");

/// An `unlock` that is neither a list nor an object with a string `context`.
pub static UNLOCK_WITHOUT_LIST_OR_CONTEXT: CodeDef<1> = CodeDef::new(
    "recipe/unlock_without_list_or_context",
    Kind::WrongType,
    Class::KeptError,
    "recipe {} has an unlock that is neither a list nor a context",
)
.game("Recipe: {} has an unlocking element but does not contain an ingredient list or and unlocking context")
.was("InvalidType");

/// An `unlock` context the game does not know.
pub static MALFORMED_UNLOCK_CONTEXT: CodeDef<1> = CodeDef::new(
    "recipe/malformed_unlock_context",
    Kind::InvalidValue,
    Class::KeptError,
    "recipe {} has an unknown unlock context",
)
.game("Recipe: {} has a malformed unlocking context")
.was("InvalidValue");

/// A shaped or shapeless recipe from 1.20.10 without `unlock`.
pub static UNLOCK_REQUIRED: CodeDef<0> = CodeDef::new(
    "recipe/unlock_required",
    Kind::MissingRequired,
    Class::Error,
    "no unlock from format 1.20.10",
)
.game("1.20+ Recipes require unlock data")
.was("MissingRequiredField");

// ---- shaped recipes ----

/// A pattern larger than 3x3: the arguments are its width and height.
pub static PATTERN_TOO_LARGE: CodeDef<2> = CodeDef::new(
    "recipe/pattern_too_large",
    Kind::InvalidValue,
    Class::Error,
    "a pattern of {} by {}, larger than 3x3",
)
.game("Adding a recipe larger than 3x3: {} width, {} height")
.was("InvalidValue");

/// A `key` entry that is no valid ingredient.
pub static KEY_INGREDIENT_MISSING: CodeDef<1> = CodeDef::new(
    "recipe/key_ingredient_missing",
    Kind::Closing,
    Class::Error,
    "recipe {} has an invalid key ingredient",
)
.game("Recipe for: {} is missing (unknown) ingredient")
.was("DocumentRejected");

/// A pattern of spaces only.
pub static EMPTY_RECIPE: CodeDef<0> = CodeDef::new(
    "recipe/empty_recipe",
    Kind::InvalidValue,
    Class::Error,
    "a pattern of spaces only",
)
.game("Recipes::addShapedRecipe: adding an empty recipe!")
.was("InvalidValue");

/// A pattern none of whose symbols has a key.
pub static NO_VALID_KEY: CodeDef<1> = CodeDef::new(
    "recipe/no_valid_key",
    Kind::InvalidValue,
    Class::Error,
    "no symbol of recipe {}'s pattern has a key",
)
.game("Recipes::addShapedRecipe: '{}' doesn't have a single valid key in its pattern!")
.was("InvalidValue");

// ---- material reduction recipes ----

/// More outputs than a material reduction recipe takes: the arguments are the identifier and the
/// maximum.
pub static REDUCTION_TOO_MANY_OUTPUTS: CodeDef<2> = CodeDef::new(
    "recipe/reduction_too_many_outputs",
    Kind::InvalidValue,
    Class::Error,
    "material reduction recipe {} has more than {} outputs",
)
.game("Material Reduction Recipe: {} has too many outputs, the max is {}.")
.was("InvalidValue");

/// An invalid material reduction input.
pub static REDUCTION_INVALID_INPUT: CodeDef<1> = CodeDef::new(
    "recipe/reduction_invalid_input",
    Kind::Closing,
    Class::Error,
    "material reduction recipe {} has an invalid input",
)
.game("Material Reduction Recipe: {} has an invalid input item")
.was("DocumentRejected");

/// An invalid material reduction output.
pub static REDUCTION_INVALID_OUTPUT: CodeDef<1> = CodeDef::new(
    "recipe/reduction_invalid_output",
    Kind::Closing,
    Class::Error,
    "material reduction recipe {} has an invalid output",
)
.game("Material Reduction Recipe: {} has an invalid output item")
.was("DocumentRejected");

// ---- brewing container recipes ----

/// An invalid brewing container input.
pub static BREWING_INVALID_INPUT: CodeDef<1> = CodeDef::new(
    "recipe/brewing_invalid_input",
    Kind::Closing,
    Class::Error,
    "brewing container recipe {} has an invalid input",
)
.game("Brewing Container Recipe: {} has an invalid input item")
.was("DocumentRejected");

/// An invalid brewing container reagent.
pub static BREWING_INVALID_REAGENT: CodeDef<1> = CodeDef::new(
    "recipe/brewing_invalid_reagent",
    Kind::Closing,
    Class::Error,
    "brewing container recipe {} has an invalid reagent",
)
.game("Brewing Container Recipe: {} has an invalid reagent item")
.was("DocumentRejected");

/// An invalid brewing container output.
pub static BREWING_INVALID_OUTPUT: CodeDef<1> = CodeDef::new(
    "recipe/brewing_invalid_output",
    Kind::Closing,
    Class::Error,
    "brewing container recipe {} has an invalid output",
)
.game("Brewing Container Recipe: {} has an invalid output item")
.was("DocumentRejected");

// ---- smithing recipes ----

/// A smithing transform recipe whose tags are not `smithing_table` alone: the arguments are the
/// identifier and `smithing_table`.
pub static TRANSFORM_INVALID_TAG: CodeDef<2> = CodeDef::new(
    "recipe/transform_invalid_tag",
    Kind::Closing,
    Class::Error,
    "smithing transform recipe {} has a tag other than {}",
)
.game("Smithing Transform Recipe: {} has an invalid tag. They only supported one is {}.")
.was("DocumentRejected");

/// An invalid smithing transform template.
pub static TRANSFORM_INVALID_TEMPLATE: CodeDef<1> = CodeDef::new(
    "recipe/transform_invalid_template",
    Kind::Closing,
    Class::Error,
    "smithing transform recipe {} has an invalid template",
)
.game("Smithing Transform Recipe: {} has an invalid template item.")
.was("DocumentRejected");

/// An invalid smithing transform base.
pub static TRANSFORM_INVALID_BASE: CodeDef<1> = CodeDef::new(
    "recipe/transform_invalid_base",
    Kind::Closing,
    Class::Error,
    "smithing transform recipe {} has an invalid base",
)
.game("Smithing Transform Recipe: {} has an invalid base item.")
.was("DocumentRejected");

/// An invalid smithing transform addition.
pub static TRANSFORM_INVALID_ADDITION: CodeDef<1> = CodeDef::new(
    "recipe/transform_invalid_addition",
    Kind::Closing,
    Class::Error,
    "smithing transform recipe {} has an invalid addition",
)
.game("Smithing Transform Recipe: {} has an invalid addition item.")
.was("DocumentRejected");

/// An invalid smithing transform result.
pub static TRANSFORM_INVALID_RESULT: CodeDef<1> = CodeDef::new(
    "recipe/transform_invalid_result",
    Kind::Closing,
    Class::Error,
    "smithing transform recipe {} has an invalid result",
)
.game("Smithing Transform Recipe: {} has an invalid result item.")
.was("DocumentRejected");

/// A smithing trim recipe whose tags are not `smithing_table` alone: the arguments are the
/// identifier and `smithing_table`.
pub static TRIM_INVALID_TAG: CodeDef<2> = CodeDef::new(
    "recipe/trim_invalid_tag",
    Kind::Closing,
    Class::Error,
    "smithing trim recipe {} has a tag other than {}",
)
.game("Smithing Trim Recipe: {} has an invalid tag. They only supported one is {}.")
.was("DocumentRejected");

/// A smithing trim recipe in a world before Trails & Tales (1.20).
pub static TRIM_BEFORE_TRAILS_AND_TALES: CodeDef<1> = CodeDef::new(
    "recipe/trim_before_trails_and_tales",
    Kind::Closing,
    Class::Error,
    "smithing trim recipe {} in a world before 1.20",
)
.game("Smithing Trim Recipe: {} is invalid. Smithing Trim Recipes are only supported from version 1.20 onwards.")
.was("DocumentRejected");

/// Every definition of the family, for the registry of ids.
pub static ALL: &[&CodeMeta] = &[
    JSON_ERROR.meta(),
    ROOT_NOT_OBJECT.meta(),
    NO_IDENTIFIER.meta(),
    DUPLICATE_IDENTIFIER.meta(),
    MISSING_VERSION_TAG.meta(),
    NO_TAGS.meta(),
    SMITHING_TABLE_TAG_MISUSED.meta(),
    DATA_NOT_MOLANG.meta(),
    COUNT_NOT_ALLOWED.meta(),
    NO_ITEM_NAME.meta(),
    NO_ITEM_OR_TAG.meta(),
    ITEM_MISSING.meta(),
    ITEM_INVALID.meta(),
    INVALID_AUX.meta(),
    EMPTY_NAMESPACE.meta(),
    EXCESSIVE_TOKENS.meta(),
    RESULT_MALFORMED.meta(),
    NO_RESULT_ITEMS.meta(),
    MULTIPLE_OUTPUTS.meta(),
    EMPTY_UNLOCK.meta(),
    INVALID_UNLOCK_INGREDIENT.meta(),
    UNLOCK_WITHOUT_LIST_OR_CONTEXT.meta(),
    MALFORMED_UNLOCK_CONTEXT.meta(),
    UNLOCK_REQUIRED.meta(),
    PATTERN_TOO_LARGE.meta(),
    KEY_INGREDIENT_MISSING.meta(),
    EMPTY_RECIPE.meta(),
    NO_VALID_KEY.meta(),
    REDUCTION_TOO_MANY_OUTPUTS.meta(),
    REDUCTION_INVALID_INPUT.meta(),
    REDUCTION_INVALID_OUTPUT.meta(),
    BREWING_INVALID_INPUT.meta(),
    BREWING_INVALID_REAGENT.meta(),
    BREWING_INVALID_OUTPUT.meta(),
    TRANSFORM_INVALID_TAG.meta(),
    TRANSFORM_INVALID_TEMPLATE.meta(),
    TRANSFORM_INVALID_BASE.meta(),
    TRANSFORM_INVALID_ADDITION.meta(),
    TRANSFORM_INVALID_RESULT.meta(),
    TRIM_INVALID_TAG.meta(),
    TRIM_BEFORE_TRAILS_AND_TALES.meta(),
];
