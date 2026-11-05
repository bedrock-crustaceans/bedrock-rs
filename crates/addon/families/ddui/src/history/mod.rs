//! Data-driven UI documents as written, and the chains that read them.
//!
//! Roots and compositions have one struct each, read at `format_version` 1.21.130 exactly (the
//! client's loaders have one version each and end at it), so neither chain has a step and
//! neither kind has a beta struct. What changes between engine versions is the control
//! [`Catalogue`](super::Catalogue), without a format bump. Form styles (engine 1.26.60 and
//! later) have the beta struct alone.

use super::{UiComposition, UiRoot, UiStyle};
use bedrock_addon_core::versioning::Single;

/// The `minecraft:ui-root` family of the version-chain machinery (hidden: only the loader
/// names it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct RootFamily;

/// The `minecraft:ui-composition` family.
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct CompositionFamily;

/// A root parsed by the struct its `format_version` selects: there is one (format 1.21.130
/// exactly).
pub type RootAny = Single<RootFamily>;

/// A composition parsed by the struct its `format_version` selects: there is one (format
/// 1.21.130 exactly).
pub type CompositionAny = Single<CompositionFamily>;

bedrock_addon_core::impl_no_molang!(UiRoot, UiComposition);

/// The `minecraft:ui-style` family.
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct StyleFamily;

bedrock_addon_core::version_chain! {
    family = StyleFamily;
    /// A form style as parsed: the beta struct is the only one.
    #[derive(Clone, Debug, PartialEq)]
    pub enum StyleAny {}
    beta =
        /// `"format_version": "beta"`.
        Beta(UiStyle);
}
