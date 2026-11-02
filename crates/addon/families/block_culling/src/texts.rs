//! Vanilla's content-log texts of the block-culling loader, each with `{}` where vanilla's `%s` /
//! `%d` / `{}` goes.

use bedrock_addon_core::diagnostic::VanillaText;

// The texts of rule 6 (the cross-checks against blocks and geometry) are the link stage's.

/// Rules 1 and 2: the file's path.
pub(super) const MISSING_OBJECT: &str =
    "Failed to parse block culling file '{}'. Missing object \"minecraft:block_culling_rules\"";
/// Rules 2 and 3: the file's path.
pub(super) const FAILED_TO_PARSE: &str = "Failed to parse block culling file '{}'.";
/// Rules 4 and 5: the file's path.
pub(super) const FAILED_INTERNAL: &str =
    "Failed to parse block culling file '{}' into internal format";
// Rule 3's generic texts (an unknown member, a missing member, an enum, a wrong type, `cube`
// below 0) are the game's shared ones: `diagnostic::cereal`.
/// Rule 4.
pub(super) const INVALID_IDENTIFIER: &str = "Invalid identifier";
/// Rule 4.
pub(super) const NEEDS_NAMESPACE: &str = "Identifier must have a namespace";
/// Rule 4.
pub(super) const MINECRAFT_NAMESPACE: &str = "Identifier not allowed to use Minecraft namespace";
/// Rule 4.
pub(super) const ILLEGAL_CHARACTERS: &str = "Identifier cannot have illegal characters";
/// Rule 4.
pub(super) const NAMESPACE_INVALID: &str = "Namespace is invalid";
/// Rule 4.
pub(super) const NAME_INVALID: &str = "Identifier name is invalid";
/// Rule 4: the identifier.
pub(super) const IDENTIFIER_FAILED: &str =
    "Identifier '{}' failed validation when registering block culling json";
/// Rule 5.1: the rule's index, the bone.
pub(super) const CUBE_WITHOUT_FACE: &str =
    "Rule {} with geometry part '{}' had a cube index but no face specified";
/// Rule 5.2: the rule's index, the bone.
pub(super) const FACE_WITHOUT_CUBE: &str =
    "Rule {} with geometry part '{}' had a face specified, but no cube index";
/// Rule 5.3: the rule's index, the bone, the cube index (-1 without one), the face
/// (`Unknown facing` without one).
pub(super) const DUPLICATE_PART: &str =
    "Rule {} had geometry part '{}' cube index {} face index '{}' that matches an earlier rule";

/// `template` with `args`.
pub(super) fn formatted(template: &'static str, args: &[&str]) -> VanillaText {
    VanillaText::Formatted {
        template,
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
    }
}

/// A culling text after its breadcrumb, as vanilla prints a parse error: `-> rules[0] ->
/// direction: <text>`.
pub(super) fn at_breadcrumb(breadcrumb: &str, text: &VanillaText) -> VanillaText {
    VanillaText::Formatted {
        template: "{}: {}",
        args: vec![breadcrumb.to_owned(), text.to_string()],
    }
}

/// Every template of this module with the number of its arguments, for the arity test.
#[cfg(test)]
const TEMPLATES: &[(&str, usize)] = &[
    (MISSING_OBJECT, 1),
    (FAILED_TO_PARSE, 1),
    (FAILED_INTERNAL, 1),
    (INVALID_IDENTIFIER, 0),
    (NEEDS_NAMESPACE, 0),
    (MINECRAFT_NAMESPACE, 0),
    (ILLEGAL_CHARACTERS, 0),
    (NAMESPACE_INVALID, 0),
    (NAME_INVALID, 0),
    (IDENTIFIER_FAILED, 1),
    (CUBE_WITHOUT_FACE, 2),
    (FACE_WITHOUT_CUBE, 2),
    (DUPLICATE_PART, 4),
];

#[cfg(test)]
mod tests {
    use super::TEMPLATES;

    #[test]
    fn every_template_has_the_number_of_arguments_its_rule_gives() {
        for (template, arguments) in TEMPLATES {
            assert_eq!(template.matches("{}").count(), *arguments, "{template}");
        }
    }
}
