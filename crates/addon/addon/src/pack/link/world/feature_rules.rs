//! The feature a rule places.

use super::super::{At, Findings, LinkCx, Target, invalid, kinds, unresolved};
use crate::diagnostic::{JsonPath, Severity, VanillaText};
use crate::families::feature::Feature;
use crate::families::feature_rules::PlacementPass;
use crate::pack::PackSide;

/// vanilla's text for a feature of another type in the pregeneration pass.
const ONLY_CAVE_CARVERS: &str =
    "\"cave_carver_feature\" is the only valid feature in \"pregeneration_pass\" placement pass.";

/// vanilla's text for a rule that names a feature of an internal type.
const INTERNAL_FEATURE: &str = "Feature rule {} can't place internal feature type.";

/// The path of a rule's `places_feature`.
fn places_feature() -> JsonPath {
    let mut path = JsonPath::new();
    for key in ["minecraft:feature_rules", "description", "places_feature"] {
        path.push(key);
    }
    path
}

/// The feature a rule places.
///
/// - A name no feature has is an `UnresolvedReference` information, a lint: the game reserves
///   the name and says nothing.
/// - A rule of a pack that is not the game's naming a feature of an internal type is an
///   `InvalidReference` kept error with vanilla's text; the game skips the rule.
/// - A rule of the pregeneration pass naming a feature that is no cave carver is an
///   `InvalidReference` kept error with vanilla's text; the game skips the rule.
///
/// One finding per rule: the game tests the internal type first.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    let behavior = cx.behavior();
    let features = behavior.features();
    for (name, document) in behavior.feature_rules() {
        let rule = document.value();
        let id = &rule.description.places_feature;
        let origin = document.origin();
        match cx.feature(id) {
            Target::Missing(hint) => {
                if cx.lints() {
                    let at = At::new(origin, places_feature(), Severity::Info);
                    out.push(unresolved(at, kinds::FEATURE, id.as_str(), hint));
                }
            }
            Target::Defined(_) => {
                let Some(placed) = features.get(id) else {
                    continue;
                };
                // The app's own definitions are the game's.
                let of_the_game = origin.pack.is_app_data()
                    || cx
                        .stack_pack(PackSide::Behavior, &origin.pack)
                        .is_some_and(|pack| pack.context.is_base_game_pack);
                let text = if placed.value().is_internal() && !of_the_game {
                    VanillaText::Formatted {
                        template: INTERNAL_FEATURE,
                        args: vec![name.as_str().to_owned()],
                    }
                } else if rule.conditions.pass() == Some(PlacementPass::Pregeneration)
                    && !is_cave_carver(placed.value())
                {
                    VanillaText::Fixed(ONLY_CAVE_CARVERS)
                } else {
                    continue;
                };
                // Both lines are logged at the error level and the rule is skipped but stays
                // registered: a kept error.
                let at = At::new(origin, places_feature(), Severity::Error).with_text(text);
                out.push(invalid(at, kinds::FEATURE, id.as_str()).into_kept());
            }
            // A feature of the game outside the stack: its type is not known here.
            _ => {}
        }
    }
}

/// Whether the feature is a cave carver: the one kind the pregeneration pass places.
// The cave feature checks its own placement; the nether and the underwater carver derive from
// it.
fn is_cave_carver(feature: &Feature) -> bool {
    matches!(
        feature,
        Feature::CaveCarver(_) | Feature::NetherCaveCarver(_) | Feature::UnderwaterCaveCarver(_)
    )
}
