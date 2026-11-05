//! Camera aim assist (`AimAssist`, `AimAssistPreset`, `AimAssistCategories`): the two document
//! kinds below `aim_assist/` of a behaviour pack, presets (`aim_assist/presets/`,
//! `minecraft:aim_assist_preset`) and category files (`aim_assist/categories/`,
//! `minecraft:aim_assist_categories`), loaded together as the `AimAssist` group.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;
pub use categories::{AimAssistCategories, AimAssistCategory, Priorities};
#[doc(hidden)]
pub use link::{Targets, link, link_camera_presets};
pub use preset::{AimAssistPreset, ExclusionSettings};

mod categories;
mod checks;
mod link;
mod load;
mod new;
mod parse;
mod preset;
mod texts;

#[cfg(test)]
mod tests;

#[doc(hidden)]
pub use load::LOADER;

use bedrock_addon_core::diagnostic::Diagnostic;
use bedrock_addon_core::document::Registry;

/// The aim-assist documents of the behaviour packs (`bp.aim_assist()`): two collections loaded
/// and used together.
///
/// Presets are keyed by their `identifier`, categories by their `name`; the highest-priority
/// pack's definition wins, whole. Vanilla reads only the files directly in `aim_assist/presets/`
/// and `aim_assist/categories/` whose extension is exactly `json`. A preset file holds one
/// preset; a categories file holds several categories and loads all of them or none.
#[derive(Debug)]
#[non_exhaustive]
pub struct AimAssist {
    /// The aim-assist presets (`aim_assist/presets/*.json`).
    pub presets: Registry<AimAssistPreset>,
    /// The aim-assist categories (`aim_assist/categories/*.json`), one entry per category.
    pub categories: Registry<AimAssistCategory>,
}

impl AimAssist {
    /// Every finding of the two collections, categories first (vanilla's load order).
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.categories
            .diagnostics()
            .chain(self.presets.diagnostics())
    }
}
