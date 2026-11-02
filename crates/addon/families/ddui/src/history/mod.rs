//! Data-driven UI documents as written, and the chains that read them.
//!
//! Roots and compositions have one struct each, read at `format_version` 1.21.130 exactly (the
//! client's loaders have one version each and end at it), so neither chain has a step and
//! neither kind has a beta struct. What changes between engine versions is the control
//! [`Catalogue`](super::Catalogue), without a format bump. Form styles (engine 1.26.60 and
//! later) have the beta struct alone.

use super::{UiComposition, UiRoot, UiStyle};

/// The `minecraft:ui-root` family of the version-chain machinery (hidden: only the loader
/// names it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct RootFamily;

/// The `minecraft:ui-composition` family.
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct CompositionFamily;

bedrock_addon_core::version_chain! {
    family = RootFamily;
    /// A root parsed by the struct its `format_version` selects: there is one.
    #[derive(Clone, Debug, PartialEq)]
    pub enum RootAny {
        /// Format 1.21.130 exactly: the only root struct.
        V1_21_130(UiRoot),
    }
    molang = none;
}

bedrock_addon_core::version_chain! {
    family = CompositionFamily;
    /// A composition parsed by the struct its `format_version` selects: there is one.
    #[derive(Clone, Debug, PartialEq)]
    pub enum CompositionAny {
        /// Format 1.21.130 exactly: the only composition struct.
        V1_21_130(UiComposition),
    }
    molang = none;
}

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
