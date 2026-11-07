//! Vanilla's content-log line the link pass raises for a loot entry's item name, byte for byte
//! with `%s` written `{}`: the one template of the family that its own [`codes`](crate::codes)
//! do not carry yet, because its raise site is the link pass's
//! (`addon/src/pack/link/content/loot_item.rs`), which still takes a legacy text.

use bedrock_addon_core::diagnostic::VanillaText;

/// A vanilla text with `N` arguments.
#[derive(Clone, Copy, Debug)]
pub struct Template<const N: usize>(pub(crate) &'static str);

/// The text of `template` with `args`.
pub fn formatted<const N: usize>(template: Template<N>, args: [&str; N]) -> VanillaText {
    if N == 0 {
        return VanillaText::Fixed(template.0);
    }
    VanillaText::Formatted {
        template: template.0,
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
    }
}

// Reported by the link pass, which resolves item names.
pub const INVALID_ITEM: Template<1> = Template("{} is not a valid item name.");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatted_fills_the_placeholder() {
        assert_eq!(INVALID_ITEM.0.matches("{}").count(), 1);
        assert_eq!(
            formatted(INVALID_ITEM, ["a:b"]).to_string(),
            "a:b is not a valid item name."
        );
    }
}
