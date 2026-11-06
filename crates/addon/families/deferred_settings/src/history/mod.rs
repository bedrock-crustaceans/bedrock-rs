//! The deferred renderer's settings as written in older format versions, and their chains.
//!
//! The game reads a settings file only when its `format_version` is exactly the version of
//! one of the payload's structs; nothing between, below or above loads, and no payload has a
//! struct for the game's experimental builds. Four payloads have older structs, each upgraded
//! step by step to the newest:
//!
//! - lighting: 1.21.40 → 1.21.60 → 1.21.70 → 1.21.80 → 1.26.0 ([`lighting`]);
//! - water: 1.21.40 → 1.21.80 → 1.21.120 → 1.26.0 → 1.26.20 ([`water`]);
//! - atmosphere: 1.21.40 → 1.26.20 ([`atmosphere`]);
//! - colour grading: 1.21.40 → 1.21.90 ([`color_grading`]).
//!
//! Cubemaps (1.21.130), shadows (1.21.80), PBR fallback (1.21.40) and local lights (1.21.120)
//! have one struct each.

/// Declares a payload's hidden chain family, its history (the `version_chain!` enum, or `Single`
/// for a payload of one struct) and its `upgrade`.
macro_rules! chain {
    (
        latest = $latest:ty;
        $(#[$meta:meta])*
        enum $history:ident { $( $(#[$vmeta:meta])* $variant:ident($ty:ty) ),+ $(,)? }
    ) => {
        /// The payload's family of the version-chain machinery (hidden: only the loaders name
        /// it).
        #[doc(hidden)]
        #[derive(Clone, Copy, Debug)]
        pub struct Family;

        bedrock_addon_core::version_chain! {
            family = Family;
            $(#[$meta])*
            #[derive(Clone, Debug, PartialEq)]
            pub enum $history {
                $( $(#[$vmeta])* $variant($ty), )+
            }
            molang = none;
        }

        /// Runs the remaining upgrade steps of a document `parse_as_written` gave: the same
        /// result `parse` gives for its file. The document holds the origin, so the upgrade
        /// needs nothing else.
        ///
        /// # Errors
        ///
        /// Returns [`Rejected`](bedrock_addon_core::document::Rejected) when the upgrade step, or the
        /// checks that follow it, find an error that stops the document from loading.
        pub fn upgrade(
            document: ::bedrock_addon_core::document::Document<$history>,
            cx: &::bedrock_addon_core::load::LoadContext,
        ) -> Result<::bedrock_addon_core::document::Document<$latest>, ::bedrock_addon_core::document::Rejected> {
            $crate::file::upgrade::<Family>(document, cx)
        }
    };
    (
        latest = $latest:ty;
        $(#[$meta:meta])*
        single $history:ident;
    ) => {
        /// The payload's family of the version-chain machinery (hidden: only the loaders name
        /// it).
        #[doc(hidden)]
        #[derive(Clone, Copy, Debug)]
        pub struct Family;

        $(#[$meta])*
        pub type $history = ::bedrock_addon_core::versioning::Single<Family>;

        ::bedrock_addon_core::impl_no_molang!($latest);

        /// Runs the checks of a document `parse_as_written` gave: the same result `parse` gives
        /// for its file (one struct, so no upgrade step).
        ///
        /// # Errors
        ///
        /// Returns [`Rejected`](bedrock_addon_core::document::Rejected) when the checks find an
        /// error that stops the document from loading.
        pub fn upgrade(
            document: ::bedrock_addon_core::document::Document<$history>,
            cx: &::bedrock_addon_core::load::LoadContext,
        ) -> Result<::bedrock_addon_core::document::Document<$latest>, ::bedrock_addon_core::document::Rejected> {
            $crate::file::upgrade::<Family>(document, cx)
        }
    };
}

pub mod atmosphere;
pub mod color_grading;
pub mod lighting;
pub mod water;

/// Cubemap settings: one struct, at 1.21.130.
pub mod cubemap {
    chain! {
        latest = crate::CubemapSettings;
        /// A cubemap-settings file as parsed: the one struct, at 1.21.130.
        single CubemapHistory;
    }
}

/// The shadow style: one struct, at 1.21.80.
pub mod shadows {
    chain! {
        latest = crate::ShadowSettings;
        /// A shadow-settings file as parsed: the one struct, at 1.21.80.
        single ShadowsHistory;
    }
}

/// The PBR fallback values: one struct, at 1.21.40.
pub mod pbr_fallback {
    chain! {
        latest = crate::PbrFallbackSettings;
        /// A PBR-fallback file as parsed: the one struct, at 1.21.40.
        single PbrFallbackHistory;
    }
}

/// The local lights: one struct, at 1.21.120.
pub mod local_lighting {
    chain! {
        latest = crate::LocalLightSettings;
        /// A local-light file as parsed: the one struct, at 1.21.120.
        single LocalLightingHistory;
    }
}
