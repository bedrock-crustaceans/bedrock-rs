//! Vanilla's content-log texts of the texture loaders: `{}` is where `VanillaText::Formatted`
//! puts each argument. Each family has its own texts; the builders are shared.

use bedrock_addon_core::diagnostic::VanillaText;

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
#[doc(hidden)]
pub fn assert_arity(templates: &[(&str, usize)]) {
    for (template, arguments) in templates {
        assert_eq!(template.matches("{}").count(), *arguments, "{template}");
    }
}
