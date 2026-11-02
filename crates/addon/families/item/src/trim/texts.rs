//! Vanilla's content-log texts of the trim registries. `{}` is
//! where `VanillaText::Formatted` puts each argument.

/// A received pattern whose template item is already a key (`{}` is the item name). The game
/// logs it at the error level of the item log area.
pub(super) const DUPLICATE_TEMPLATE: &str =
    "Overwriting pattern template. Duplicate entries with template item: \"{}\"";
