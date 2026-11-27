//! Per-adjacent-version block-schema diffs, backed by the generated tables under
//! [`crate::generated`] (architecture decision 12).
//!
//! Hand-written: this module defines the shapes the generator emits data into. No diff
//! data lives here -- that is entirely what `crate::generated` holds, and it exists only
//! when the `generated-palettes` feature is enabled.
//!
//! What one [`VersionDiff`] answers: for one adjacent pair in the pinned version ladder,
//! which block names were added, which were removed, which of those removals the
//! hand-authored rename supplement (`xtask/dumps/bedrock-samples/renames.json`) confirms
//! are actually the same block continuing under a new name, which blocks kept their name
//! but changed their declared property set, and which properties kept their name but
//! changed their declared value domain (or type). A name that disappears in one diff and a
//! different name that appears in the same diff are never merged into a rename unless
//! [`VersionDiff::renames`] says so explicitly -- see the generator's own doc comment
//! (`xtask/src/codegen/palette/diff.rs`) for why guessing from the coincidence is refused.

/// One confirmed rename that applied at this diff's step, sourced entirely from the
/// hand-authored rename supplement -- never inferred from a removed/added name pair on
/// its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenameChange {
    pub from: &'static str,
    pub to: &'static str,
}

/// One block, present at both versions this diff spans, whose declared property *set*
/// changed -- not its value domains, which are tracked per-property in
/// [`PropertyChange`] instead, since one property can be shared by many blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockChange {
    pub block: &'static str,
    /// Sorted.
    pub properties_added: &'static [&'static str],
    /// Sorted.
    pub properties_removed: &'static [&'static str],
}

/// How one property's value domain, or its type, changed between the two versions a
/// [`VersionDiff`] spans, for a property name declared at both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyValueDelta {
    /// The property's declared type itself changed (e.g. `int` -> `string`) --
    /// unobserved in the pinned dumps to date, but tracked as a distinct fact rather than
    /// folded into `Int`/`Str` so a genuinely surprising future dump cannot be
    /// misrepresented as an ordinary value-domain change.
    KindChanged {
        from: &'static str,
        to: &'static str,
    },
    /// Both versions declare this property as `int`; its integer domain changed.
    Int {
        added: &'static [i64],
        removed: &'static [i64],
    },
    /// Both versions declare this property as `string`; its string domain changed.
    Str {
        added: &'static [&'static str],
        removed: &'static [&'static str],
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropertyChange {
    pub property: &'static str,
    pub delta: PropertyValueDelta,
}

/// The full diff between two adjacent pinned versions in the block-palette ladder.
#[derive(Debug, Clone, Copy)]
pub struct VersionDiff {
    pub from_tag: &'static str,
    pub to_tag: &'static str,
    pub from_version: (u8, u8, u8),
    pub to_version: (u8, u8, u8),
    /// Sorted. A name declared at `to_version` with no counterpart at `from_version`
    /// that `renames` does not account for.
    pub blocks_added: &'static [&'static str],
    /// Sorted. A name declared at `from_version` with no counterpart at `to_version`
    /// that `renames` does not account for.
    pub blocks_removed: &'static [&'static str],
    /// Sorted by [`RenameChange::from`].
    pub renames: &'static [RenameChange],
    /// Sorted by [`BlockChange::block`].
    pub block_changes: &'static [BlockChange],
    /// Sorted by [`PropertyChange::property`].
    pub property_changes: &'static [PropertyChange],
}

impl VersionDiff {
    /// The name this block continues under at `to_version`, applying a confirmed rename
    /// if one covers `name`, or `name` unchanged if none does. Does not check whether the
    /// resulting name is actually declared at `to_version` -- that is
    /// [`crate::palette::VersionPalette::block`]'s question, asked separately by the
    /// upgrade walker in `crate`'s root.
    pub fn resolve_name<'a>(&self, name: &'a str) -> &'a str {
        self.renames
            .iter()
            .find(|r| r.from == name)
            .map_or(name, |r| r.to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIFF: VersionDiff = VersionDiff {
        from_tag: "vA",
        to_tag: "vB",
        from_version: (1, 0, 0),
        to_version: (1, 1, 0),
        blocks_added: &["minecraft:new"],
        blocks_removed: &["minecraft:gone"],
        renames: &[RenameChange {
            from: "minecraft:old_name",
            to: "minecraft:new_name",
        }],
        block_changes: &[],
        property_changes: &[],
    };

    #[test]
    fn resolve_name_applies_a_confirmed_rename() {
        assert_eq!(
            DIFF.resolve_name("minecraft:old_name"),
            "minecraft:new_name"
        );
    }

    #[test]
    fn resolve_name_passes_through_an_unrenamed_name() {
        assert_eq!(DIFF.resolve_name("minecraft:stone"), "minecraft:stone");
    }
}
