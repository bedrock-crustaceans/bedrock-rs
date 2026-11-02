//! Vanilla's content-log texts of the loot walkers, byte
//! for byte with `%s` / `%d` written `{}`. Each template carries its argument count in its type,
//! so a finding cannot be built with the wrong number of arguments.

use bedrock_addon_core::diagnostic::VanillaText;

/// A vanilla text with `N` arguments.
#[derive(Clone, Copy, Debug)]
pub struct Template<const N: usize>(pub(crate) &'static str);

/// The text of `template` with `args`.
#[must_use]
pub fn formatted<const N: usize>(template: Template<N>, args: [&str; N]) -> VanillaText {
    if N == 0 {
        return VanillaText::Fixed(template.0);
    }
    VanillaText::Formatted {
        template: template.0,
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
    }
}

/// The text of a template without arguments.
pub(crate) fn fixed(template: Template<0>) -> VanillaText {
    VanillaText::Fixed(template.0)
}

pub(crate) const INVALID_TABLE: Template<1> = Template("Loot table for {} is invalid.");
pub(crate) const ENTRIES_ARRAY: Template<0> =
    Template("There must be an \"entries\" array inside the loot table pool.");
pub(crate) const ENTRY_OBJECT: Template<0> = Template(
    "Every entry of \"entries\" must be a JSON object - it has to start with an \"{\" and end with a \"}\"",
);
pub(crate) const ENTRY_TYPE: Template<1> = Template(
    "Every entry of \"entries\" must specify a type of either \"item\", \"loot_table\", or \"empty\". Given: {}",
);
// Reported by the link pass, which resolves item names.
pub const INVALID_ITEM: Template<1> = Template("{} is not a valid item name.");
pub(crate) const FUNCTION_OBJECT: Template<0> = Template(
    "Error loading loot table \"functions\" as it is not a valid Json object. \"functions\" must be key: value pairs",
);
pub(crate) const UNSUPPORTED_FUNCTION: Template<1> = Template("Unsupported looting function: {}");
pub(crate) const UCF_FUNCTION: Template<1> =
    Template("Unsupported looting function outside of Upcoming Creator Features Experiment: {}");
pub(crate) const DYNAMIC_PROPERTIES: Template<0> = Template(
    "Error loading loot table \"carry_over_block_entity_data\" dynamic_properties as it is not a bool",
);
pub(crate) const CONDITION_OBJECT: Template<0> = Template(
    "Error loading loot table \"conditions\" as it is not a valid Json object. \"conditions\" must be key: value pairs",
);
// The game logs this line for a condition whose operator is none of these.
pub(crate) const CONDITION_OPERATOR: Template<1> = Template(
    "Invalid loot condition operator \"{}\". Expected \"==\", \"equals\", \"!=\", or \"not\".",
);
// The game writes this line to its developer log, not the content log.
pub(crate) const UNKNOWN_CONDITION: Template<1> =
    Template("There is unknown or supressed condition[{}]");
pub(crate) const BANNER_TYPE: Template<1> = Template("Invalid banner type input: {}");
pub(crate) const BANNER_BASE_COLOR: Template<1> = Template("Invalid banner base_color input: {}");
pub(crate) const BANNER_NON_DEFAULT: Template<0> =
    Template("Non-default banner types cannot have custom patterns");
pub(crate) const BANNER_MAX: Template<2> =
    Template("Banner exceeds maximum number of {} patterns! Patterns given: {}");
pub(crate) const BANNER_PATTERN: Template<1> = Template("Invalid banner pattern name: {}");
pub(crate) const BANNER_PATTERN_COLOR: Template<1> =
    Template("Invalid color input in banner pattern {}");
pub(crate) const SET_DATA_KEY: Template<0> = Template(
    "Invalid json keyname for specifying the data range for set_data function. Only 'data' and 'values' are supported.",
);
pub(crate) const ENCHANT_ARRAY: Template<0> = Template(
    "Error loading \"enchant_book_for_trading\" function, \"enchantments\": should be type array",
);
pub(crate) const ENCHANT_NAME: Template<1> = Template(
    "Error loading \"enchant_book_for_trading\" function, \"name\": {} is not a valid enchantment name",
);

#[cfg(test)]
mod tests {
    use super::*;

    /// The `{}` placeholders of a template.
    fn placeholders(template: &str) -> usize {
        template.matches("{}").count()
    }

    fn arity<const N: usize>(template: Template<N>) -> (usize, usize) {
        (placeholders(template.0), N)
    }

    #[test]
    fn every_template_has_one_placeholder_per_argument() {
        let all = [
            arity(INVALID_TABLE),
            arity(ENTRIES_ARRAY),
            arity(ENTRY_OBJECT),
            arity(ENTRY_TYPE),
            arity(INVALID_ITEM),
            arity(FUNCTION_OBJECT),
            arity(UNSUPPORTED_FUNCTION),
            arity(UCF_FUNCTION),
            arity(DYNAMIC_PROPERTIES),
            arity(CONDITION_OBJECT),
            arity(UNKNOWN_CONDITION),
            arity(CONDITION_OPERATOR),
            arity(BANNER_TYPE),
            arity(BANNER_BASE_COLOR),
            arity(BANNER_NON_DEFAULT),
            arity(BANNER_MAX),
            arity(BANNER_PATTERN),
            arity(BANNER_PATTERN_COLOR),
            arity(SET_DATA_KEY),
            arity(ENCHANT_ARRAY),
            arity(ENCHANT_NAME),
        ];
        for (index, (found, declared)) in all.into_iter().enumerate() {
            assert_eq!(found, declared, "template {index}");
        }
    }

    #[test]
    fn formatted_fills_the_placeholders_in_order() {
        let text = formatted(BANNER_MAX, ["6", "7"]);
        assert_eq!(
            text.to_string(),
            "Banner exceeds maximum number of 6 patterns! Patterns given: 7"
        );
        assert_eq!(
            formatted(ENTRY_OBJECT, []).to_string(),
            "Every entry of \"entries\" must be a JSON object - it has to start with an \"{\" and end with a \"}\""
        );
    }
}
