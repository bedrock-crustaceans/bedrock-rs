//! The second table of `#[derive(Components)]`: per component row, the facts the version
//! chain's `ComponentMeta` does not carry (its typed version, its scalar shorthand, goal-ness
//! and the trigger members of an untyped row).

use crate::version::FormatVersion;

/// What a component written as a scalar is read as: `shorthand = scalar` on the row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Shorthand {
    /// A bare number, bool or string stands for the component's one value.
    Scalar,
}

/// One row of a component enum's `SCHEMA_ROWS`, sorted by name like its `TABLE`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ComponentRow {
    /// The component's JSON key.
    pub name: &'static str,
    /// `typed = ".."`: the format version from which the game reads it with its typed struct.
    pub typed: Option<FormatVersion>,
    /// `shorthand = scalar`.
    pub shorthand: Option<Shorthand>,
    /// `goal`: the row is an AI goal.
    pub goal: bool,
    /// `triggers = [..]`: trigger members of a row whose payload is no typed struct.
    pub triggers: &'static [&'static str],
}

/// The row named `name` of `rows` (sorted by name), for the derived `schema_row`.
#[doc(hidden)]
pub fn find_row(rows: &'static [ComponentRow], name: &str) -> Option<&'static ComponentRow> {
    rows.binary_search_by(|row| row.name.cmp(name))
        .ok()
        .map(|index| &rows[index])
}
