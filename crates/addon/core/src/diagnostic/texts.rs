//! Builders for the vanilla content-log texts of any loader (the texture, spawn and
//! other families use them): `{}` is where `VanillaText::Formatted` puts each argument. The
//! texts themselves are the family's own; only these builders are shared.

use super::VanillaText;

/// A vanilla text without arguments.
#[allow(
    clippy::unnecessary_wraps,
    reason = "every call site fills an `Option<VanillaText>` field"
)]
pub fn fixed(text: &'static str) -> Option<VanillaText> {
    Some(VanillaText::Formatted {
        template: text,
        args: Vec::new(),
    })
}

/// A vanilla text with arguments: `template` with each `{}` replaced by the next argument.
/// Every formatted text of the module is built here, so a test can check the arity.
#[allow(
    clippy::unnecessary_wraps,
    reason = "every call site fills an `Option<VanillaText>` field"
)]
pub fn formatted<const N: usize>(template: &'static str, args: [String; N]) -> Option<VanillaText> {
    Some(VanillaText::Formatted {
        template,
        args: args.into(),
    })
}

/// Whether every template has as many holes as its call site gives it arguments: the check of
/// a family's `TEMPLATES`.
// hidden: API not yet reviewed
#[doc(hidden)]
pub fn assert_arity(templates: &[(&str, usize)]) {
    for (template, arguments) in templates {
        assert_eq!(template.matches("{}").count(), *arguments, "{template}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The texture loaders' `fixed` is a `Formatted` text without arguments, the shape check's
    /// (`versioning::shape::fixed`) a `Fixed` one: the two render alike and are not equal.
    #[test]
    fn the_two_fixed_texts_differ_in_their_variant() {
        let text = "a vanilla text";
        assert_eq!(
            fixed(text),
            Some(VanillaText::Formatted {
                template: text,
                args: Vec::new(),
            })
        );
        assert_eq!(
            crate::versioning::shape::fixed(text),
            Some(VanillaText::Fixed(text))
        );
        assert_ne!(fixed(text), crate::versioning::shape::fixed(text));
    }
}
