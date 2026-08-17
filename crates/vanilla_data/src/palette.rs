//! Per-version block-palette lookup, backed by the generated tables under
//! [`crate::generated`] (architecture decision 12).
//!
//! Hand-written: this module defines the shapes the generator emits data into and the
//! lookup built on top of them. No block or version data lives here -- that is entirely
//! what `crate::generated` holds, and it exists only when the `generated-palettes`
//! feature is enabled.
//!
//! What a [`VersionPalette`] can answer, and what it deliberately cannot: it is
//! generated from the game's own published block-palette dump for one version, which
//! names every block and which property *names* it accepts, and separately, for each
//! property name, its type and the value domain the dump enumerates. It does not carry a
//! default value for any property -- the dump itself does not record one -- so nothing
//! here fabricates one. It also does not carry any cross-version mapping (renames,
//! property-value remaps, flattening): that needs a diff between two versions' data plus
//! rules for how to apply it, which is a later deliverable this module does not attempt.
//!
//! What it *can* check, and what calls it: [`PropertyKind::accepts`] validates one
//! property's value against the domain the dump enumerates for it, which
//! `VanillaContent::upgrade_block_state` (in the crate root) uses to reject a state
//! carrying a property the block does not declare, or a value outside its declared
//! domain, as [`bedrock_level::provider::StateUpgrade::Malformed`] rather than silently
//! accepting it.

/// The kind of value a block property holds, and the value domain the published dump
/// enumerates for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyKind {
    Bool,
    Int(&'static [i64]),
    Str(&'static [&'static str]),
}

impl PropertyKind {
    /// Whether `value` is a legal value for a property of this kind, per the domain the
    /// dump enumerates.
    ///
    /// Deliberately lenient about which NBT tag carries an integer, because the dump's
    /// `type` field is a logical bucket (`bool`/`int`/`string`) that says nothing about
    /// NBT wire width -- committing to one width here would assert a constraint the
    /// source data does not state, and reject a value whose *meaning* is in-domain over
    /// a width guess. So `Bool`/`Int` accept any of `Byte`/`Short`/`Int`/`Long`. A
    /// `Bool` is a two-valued `Int` at the NBT layer -- Bedrock has no boolean tag --
    /// so it accepts exactly `0` or `1` under the same widths. (Real palette entries
    /// have only ever been observed writing `states` values as `Int`; if a caller needs
    /// that stricter fact, it lives in the corpus tests, not in this domain check.)
    pub fn accepts(&self, value: &nbtx::Value) -> bool {
        match self {
            Self::Bool => matches!(as_i64(value), Some(0 | 1)),
            Self::Int(domain) => as_i64(value).is_some_and(|v| domain.contains(&v)),
            Self::Str(domain) => match value {
                nbtx::Value::String(s) => domain.iter().any(|d| s.as_slice() == d.as_bytes()),
                _ => false,
            },
        }
    }
}

/// Reads an NBT integer tag of any width as `i64`, or `None` for a non-integer tag.
fn as_i64(value: &nbtx::Value) -> Option<i64> {
    match *value {
        nbtx::Value::Byte(b) => Some(b as i64),
        nbtx::Value::Short(s) => Some(s as i64),
        nbtx::Value::Int(i) => Some(i as i64),
        nbtx::Value::Long(l) => Some(l),
        _ => None,
    }
}

/// One property name plus the kind and legal value domain the dump records for it.
///
/// Shared across every block in a [`VersionPalette`] that accepts it: the dump stores
/// properties this way (one table of properties, blocks reference them by name) rather
/// than repeating a property's value domain on every block that uses it, and the
/// generated data preserves that shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropertySpec {
    pub name: &'static str,
    pub kind: PropertyKind,
}

/// One block name plus the property names it accepts, at one game version.
///
/// Carries no default value for any property -- see the module docs on why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockSpec {
    pub name: &'static str,
    pub properties: &'static [&'static str],
}

/// One version's full block palette, as generated from one pinned dump.
#[derive(Debug, Clone, Copy)]
pub struct VersionPalette {
    /// The upstream release tag this was generated from, e.g. `"v1.20.10.1"`.
    pub tag: &'static str,
    /// `(major, minor, patch)` -- the same three bytes a stored palette entry's version
    /// stamp carries in the same positions (see
    /// [`bedrock_level::subchunk::BlockDef::version`]), ignoring the fourth (revision)
    /// byte, which is a same-version sub-stamp rather than a distinct game version.
    pub game_version: (u8, u8, u8),
    /// Sorted by name; generated that way, so lookup is a binary search.
    pub properties: &'static [PropertySpec],
    /// Sorted by name; generated that way, so lookup is a binary search.
    pub blocks: &'static [BlockSpec],
}

impl VersionPalette {
    /// Looks up a block by name.
    pub fn block(&self, name: &str) -> Option<&'static BlockSpec> {
        self.blocks
            .binary_search_by(|b| b.name.cmp(name))
            .ok()
            .map(|i| &self.blocks[i])
    }

    /// Looks up a property's kind and value domain by name.
    pub fn property(&self, name: &str) -> Option<&'static PropertySpec> {
        self.properties
            .binary_search_by(|p| p.name.cmp(name))
            .ok()
            .map(|i| &self.properties[i])
    }
}

/// Finds the generated palette whose [`VersionPalette::game_version`] matches
/// `(major, minor, patch)` exactly.
///
/// `None` when no generated table covers that exact version -- the caller is expected to
/// treat that the same as any other "no entry" outcome, not to fall back to a
/// neighbouring version, since nothing here knows whether the palette actually changed
/// between them.
///
/// Only compiled with the `generated-palettes` feature, since [`crate::generated`] does
/// not exist without it.
#[cfg(feature = "generated-palettes")]
pub fn palette_for_version(major: u8, minor: u8, patch: u8) -> Option<&'static VersionPalette> {
    crate::generated::PALETTES
        .iter()
        .find(|p| p.game_version == (major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bool_accepts_zero_and_one_under_any_integer_width() {
        let kind = PropertyKind::Bool;
        assert!(kind.accepts(&nbtx::Value::Byte(0)));
        assert!(kind.accepts(&nbtx::Value::Byte(1)));
        assert!(kind.accepts(&nbtx::Value::Int(1)));
        assert!(kind.accepts(&nbtx::Value::Long(0)));
        assert!(!kind.accepts(&nbtx::Value::Byte(2)));
        assert!(!kind.accepts(&nbtx::Value::String("true".into())));
    }

    #[test]
    fn int_accepts_only_values_in_its_domain_under_any_integer_width() {
        let kind = PropertyKind::Int(&[0, 1, 2, 3]);
        assert!(kind.accepts(&nbtx::Value::Byte(2)));
        assert!(kind.accepts(&nbtx::Value::Short(3)));
        assert!(kind.accepts(&nbtx::Value::Long(0)));
        assert!(!kind.accepts(&nbtx::Value::Int(99)));
        assert!(!kind.accepts(&nbtx::Value::String("2".into())));
    }

    #[test]
    fn str_accepts_only_strings_in_its_domain() {
        let kind = PropertyKind::Str(&["hanging", "standing"]);
        assert!(kind.accepts(&nbtx::Value::String("hanging".into())));
        assert!(!kind.accepts(&nbtx::Value::String("sideways".into())));
        assert!(!kind.accepts(&nbtx::Value::Byte(0)));
    }
}
