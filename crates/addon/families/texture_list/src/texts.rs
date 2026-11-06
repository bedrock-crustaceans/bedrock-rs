//! Vanilla's content-log texts of the texture lists. `{}` is where `VanillaText::Formatted`
//! puts each argument.

/// A base-game pack with a `textures/` directory and no list; the list's path, the pack.
pub(super) const NO_LIST_IN_BASE_PACK: &str = "Failed to find {} in {} for a base pack with a texture directory, falling back to pack iteration";

/// Every template and the number of arguments its call site gives it, for the arity test.
#[cfg(test)]
const TEMPLATES: &[(&str, usize)] = &[(NO_LIST_IN_BASE_PACK, 2)];

#[cfg(test)]
mod tests {
    #[test]
    fn every_template_has_as_many_holes_as_its_call_site_has_arguments() {
        bedrock_addon_core::diagnostic::texts::assert_arity(super::TEMPLATES);
    }
}
