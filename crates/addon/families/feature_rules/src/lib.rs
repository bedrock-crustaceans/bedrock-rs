//! Feature rules: `feature_rules/**/*.json` of a behaviour pack (`minecraft:feature_rules`), the
//! rules that attach a feature to the biomes it generates in. One struct covers every
//! `format_version` from 1.12.0 to the engine.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod load;
mod parse;
mod validate;
mod write;

#[cfg(test)]
mod tests;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::document::{Document, Extra, Registry, Rejected};
use bedrock_addon_core::id::{FeatureId, FeatureRuleId};
use bedrock_addon_core::json::StrictString;
use bedrock_addon_core::shared::Filter;
use bedrock_addon_core::version::FormatVersion;

pub use bedrock_addon_support::scatter::{
    CoordinateDistribution, CoordinateEvalOrder, CoordinateRange, RandomDistribution,
    ScatterChance, ScatterFraction, ScatterParams,
};

#[doc(hidden)]
pub use load::load_feature_rules;

/// The first `format_version` vanilla reads feature rules at: 1.12.0, the first schema.
pub(crate) const FLOOR: FormatVersion = FormatVersion::new(1, 12, 0);

/// A feature rule (`minecraft:feature_rules`): which feature it places, in which placement pass,
/// over which biomes, and how the positions are scattered.
///
/// JSON: `{"format_version": "1.12.0", "minecraft:feature_rules": {"description", "conditions",
/// "distribution"?}}`. A rule loads even when its identifier does not match its file name or
/// names an unknown pass (vanilla logs both and keeps the rule).
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:feature_rules")]
#[doc(alias = "feature_rules")]
pub struct FeatureRule {
    /// The rule's identifier and the feature it places (JSON `description`).
    pub description: FeatureRuleDescription,
    /// When and where the feature is placed (JSON `conditions`).
    pub conditions: FeatureRuleConditions,
    /// How the feature's positions are scattered (JSON `distribution`). Without it the rule
    /// scatters no position.
    pub distribution: Option<ScatterParams>,
    /// The document's `format_version` (the floor when absent). Not a JSON member; the writer
    /// writes it as `format_version`.
    pub source_format_version: FormatVersion,
    /// Unknown members of `minecraft:feature_rules`.
    pub extra: Extra,
}

/// `minecraft:feature_rules/description`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct FeatureRuleDescription {
    /// The rule's name, `namespace:rule_name`, where `rule_name` is the file name (JSON
    /// `identifier`).
    pub identifier: FeatureRuleId,
    /// The feature the rule places (JSON `places_feature`). A name no feature defines is
    /// reserved, not an error.
    pub places_feature: FeatureId,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:feature_rules/conditions`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct FeatureRuleConditions {
    /// The placement pass, as written (JSON `placement_pass`); [`PlacementPass`] lists the ones
    /// vanilla defines, and [`FeatureRuleConditions::pass`] reads it.
    pub placement_pass: StrictString,
    /// The biomes the rule attaches to (JSON `minecraft:biome_filter`); absent is every biome.
    #[facet(rename = "minecraft:biome_filter", default)]
    pub biome_filter: Option<Filter>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl FeatureRuleConditions {
    /// The placement pass `placement_pass` names, or `None` when it names no pass vanilla
    /// defines.
    #[must_use]
    pub fn pass(&self) -> Option<PlacementPass> {
        PlacementPass::from_name(&self.placement_pass.0)
    }
}

/// When a feature rule places its feature: the passes of world generation, in the order they
/// run. Every rule of an earlier pass is placed before any rule of a later one; within a pass
/// the order is not defined.
///
/// The game registers its twelve pass names in exactly this order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum PlacementPass {
    /// `pregeneration_pass`: cave carvers only.
    Pregeneration,
    /// `first_pass`.
    First,
    /// `before_underground_pass`.
    BeforeUnderground,
    /// `underground_pass`.
    Underground,
    /// `after_underground_pass`.
    AfterUnderground,
    /// `before_surface_pass`.
    BeforeSurface,
    /// `surface_pass`.
    Surface,
    /// `after_surface_pass`.
    AfterSurface,
    /// `before_sky_pass`.
    BeforeSky,
    /// `sky_pass`.
    Sky,
    /// `after_sky_pass`.
    AfterSky,
    /// `final_pass`.
    Final,
}

impl PlacementPass {
    /// Every pass, in the order they run.
    pub const ALL: [PlacementPass; 12] = [
        Self::Pregeneration,
        Self::First,
        Self::BeforeUnderground,
        Self::Underground,
        Self::AfterUnderground,
        Self::BeforeSurface,
        Self::Surface,
        Self::AfterSurface,
        Self::BeforeSky,
        Self::Sky,
        Self::AfterSky,
        Self::Final,
    ];

    /// The JSON name (`surface_pass`).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Pregeneration => "pregeneration_pass",
            Self::First => "first_pass",
            Self::BeforeUnderground => "before_underground_pass",
            Self::Underground => "underground_pass",
            Self::AfterUnderground => "after_underground_pass",
            Self::BeforeSurface => "before_surface_pass",
            Self::Surface => "surface_pass",
            Self::AfterSurface => "after_surface_pass",
            Self::BeforeSky => "before_sky_pass",
            Self::Sky => "sky_pass",
            Self::AfterSky => "after_sky_pass",
            Self::Final => "final_pass",
        }
    }

    /// The pass named `name`, compared exactly (case-sensitive), or `None`.
    #[must_use]
    pub fn from_name(name: &str) -> Option<PlacementPass> {
        Self::ALL.into_iter().find(|pass| pass.name() == name)
    }
}

/// The feature rules of a behaviour stack: one registry per placement pass, because vanilla keys
/// a rule by its pass and its identifier (one identifier can be live in two passes).
///
/// Packs are read from the top of the stack down and the first rule of a `(pass, identifier)`
/// pair wins; the app-data rules come last. A rule naming a pass vanilla does not define is kept
/// in [`unknown_passes`](Self::unknown_passes): it loads, but no pass places it.
#[derive(Debug)]
#[non_exhaustive]
pub struct FeatureRules {
    /// `pregeneration_pass`.
    pub pregeneration_pass: Registry<FeatureRule>,
    /// `first_pass`.
    pub first_pass: Registry<FeatureRule>,
    /// `before_underground_pass`.
    pub before_underground_pass: Registry<FeatureRule>,
    /// `underground_pass`.
    pub underground_pass: Registry<FeatureRule>,
    /// `after_underground_pass`.
    pub after_underground_pass: Registry<FeatureRule>,
    /// `before_surface_pass`.
    pub before_surface_pass: Registry<FeatureRule>,
    /// `surface_pass`.
    pub surface_pass: Registry<FeatureRule>,
    /// `after_surface_pass`.
    pub after_surface_pass: Registry<FeatureRule>,
    /// `before_sky_pass`.
    pub before_sky_pass: Registry<FeatureRule>,
    /// `sky_pass`.
    pub sky_pass: Registry<FeatureRule>,
    /// `after_sky_pass`.
    pub after_sky_pass: Registry<FeatureRule>,
    /// `final_pass`.
    pub final_pass: Registry<FeatureRule>,
    /// The rules of passes vanilla does not define, by the pass name they give, in walk order.
    pub unknown_passes: indexmap::IndexMap<String, Registry<FeatureRule>>,
    /// Files that failed to load.
    rejected: Vec<Rejected>,
    /// The findings of the files that failed and of the combine step.
    diagnostics: Diagnostics,
}

/// The rules of a [`FeatureRules`] in order, from [`FeatureRules::iter`].
pub struct Iter<'a>(Box<dyn Iterator<Item = (&'a FeatureRuleId, &'a Document<FeatureRule>)> + 'a>);

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a FeatureRuleId, &'a Document<FeatureRule>);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl std::fmt::Debug for Iter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Iter").finish_non_exhaustive()
    }
}

impl<'a> IntoIterator for &'a FeatureRules {
    type Item = (&'a FeatureRuleId, &'a Document<FeatureRule>);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

impl FeatureRules {
    /// The registry of `pass`.
    #[must_use]
    pub fn pass(&self, pass: PlacementPass) -> &Registry<FeatureRule> {
        match pass {
            PlacementPass::Pregeneration => &self.pregeneration_pass,
            PlacementPass::First => &self.first_pass,
            PlacementPass::BeforeUnderground => &self.before_underground_pass,
            PlacementPass::Underground => &self.underground_pass,
            PlacementPass::AfterUnderground => &self.after_underground_pass,
            PlacementPass::BeforeSurface => &self.before_surface_pass,
            PlacementPass::Surface => &self.surface_pass,
            PlacementPass::AfterSurface => &self.after_surface_pass,
            PlacementPass::BeforeSky => &self.before_sky_pass,
            PlacementPass::Sky => &self.sky_pass,
            PlacementPass::AfterSky => &self.after_sky_pass,
            PlacementPass::Final => &self.final_pass,
        }
    }

    /// The effective rule `id` of `pass`, or `None` if no pack defines it there.
    #[must_use]
    pub fn get(&self, pass: PlacementPass, id: &FeatureRuleId) -> Option<&Document<FeatureRule>> {
        self.pass(pass).get(id)
    }

    /// How many rules are effective, over every pass (the unknown ones included).
    pub fn len(&self) -> usize {
        self.registries().map(Registry::len).sum()
    }

    /// Whether no rule loaded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Every effective rule: the passes in the order they run, then the unknown passes; within
    /// a registry in load order.
    pub fn iter(&self) -> Iter<'_> {
        Iter(Box::new(self.registries().flat_map(Registry::iter)))
    }

    /// Files that failed to load, with their findings.
    #[must_use]
    pub fn rejected(&self) -> &[Rejected] {
        &self.rejected
    }

    /// Every finding of the family: the failed files', then each registry's (its documents' and
    /// the combine step's), the passes in order.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.diagnostics
            .iter()
            .chain(self.registries().flat_map(Registry::diagnostics))
    }

    /// The registries: the twelve passes in order, then the unknown ones.
    fn registries(&self) -> impl Iterator<Item = &Registry<FeatureRule>> + '_ {
        PlacementPass::ALL
            .into_iter()
            .map(|pass| self.pass(pass))
            .chain(self.unknown_passes.values())
    }
}
