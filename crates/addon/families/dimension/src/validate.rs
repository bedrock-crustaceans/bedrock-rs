//! The dimension validator: vanilla runs it on the upgraded 1.26.50 document whichever slice
//! parsed the file. Each failure logs its content-log text and a validation error text, and the
//! file is skipped; the crate's finding is one `InvalidValue` error carrying both texts, one per
//! line.

use super::parse::child;
use super::{Dimension, DimensionHeight, V1_26_50};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, JsonPath, Phase, VanillaText};
use bedrock_addon_core::versioning::ValidateCx;

/// The validation error text of every identifier failure (part B).
const INVALID_CUSTOM: &str = "Invalid custom dimension document.";

/// The namespace of the game's own content.
const VANILLA_NAMESPACE: &str = "minecraft";

/// The validation error text of every height failure (part A).
const INVALID_HEIGHT: &str = "Invalid dimension height.";

/// The sub-chunk height a height is a multiple of.
const SUBCHUNK_HEIGHT: i32 = 16;

/// The highest top a dimension may have.
const DIMENSION_HEIGHT_MAX: i32 = 512;

/// The rules of the validator on a loaded document, in vanilla's order: the height (part A,
/// its first failure ends it), the identifier (part B), then the generator (part C); each part
/// runs whatever the one before found.
pub(crate) fn validate(dimension: &mut Dimension, cx: &mut ValidateCx<'_>) {
    let at = child(&cx.at, &["minecraft:dimension"]);
    height_rules(dimension, &at, cx);
    identifier_rules(dimension, &at, cx);
    generator_rule(dimension, &at, cx);
}

/// Part C: a `generator_type` other than `void` is logged (`minecraft:generation:
/// generator_type unsupported`) at the content log's error level, but the validator returns no
/// validation error, so the game keeps the document and never writes "Failed to load
/// dimension": a kept error, an error-level finding that rejects nothing.
fn generator_rule(dimension: &Dimension, at: &JsonPath, cx: &mut ValidateCx<'_>) {
    if dimension.components.generation.generator().is_some() {
        return;
    }
    cx.diagnostics.push_kept_error(
        Diagnostic::error(DiagnosticCode::InvalidValue {
            text: Some(VanillaText::Fixed(
                "minecraft:generation: generator_type unsupported",
            )),
        })
        .at(child(
            at,
            &["components", "minecraft:generation", "generator_type"],
        ))
        .phase(Phase::Validate),
    );
}

/// Part A: the nether and the end take no height but `{0, 0}`; any other
/// dimension's height (or the default) is on the sub-chunk grid, positive and tops out at 512.
/// Reported at the component the height came from: `minecraft:dimension_bounds` for a file
/// below 1.26.50 (the upgrade made the height of it), else `minecraft:dimension_height`.
fn height_rules(dimension: &Dimension, at: &JsonPath, cx: &mut ValidateCx<'_>) {
    let component = if cx.original_version < V1_26_50 {
        "minecraft:dimension_bounds"
    } else {
        "minecraft:dimension_height"
    };
    let at = child(at, &["components", component]);
    let height = dimension.components.dimension_height.as_ref();
    let failure = match dimension.description.identifier.as_str() {
        "minecraft:nether" | "minecraft:the_end" => height
            .is_some_and(|h| h.min_y.0 | h.height_range.0 != 0)
            .then_some(
                "Dimension height range: nether and the end do not support custom height values",
            ),
        _ => {
            let (min_y, range) = height.map_or(Dimension::DEFAULT_HEIGHT, DimensionHeight::pair);
            let (min_y, range) = (i32::from(min_y), i32::from(range));
            if range & (SUBCHUNK_HEIGHT - 1) != 0 {
                Some("Dimension height range: height_range must be divisible by 16")
            } else if min_y & (SUBCHUNK_HEIGHT - 1) != 0 {
                Some("Dimension min y: min_y must be divisible by 16")
            } else if range <= 0 {
                Some("Dimension height range: height_range must be greater than zero")
            } else if min_y + range > DIMENSION_HEIGHT_MAX {
                Some("Dimension height range: min_y plus height_range must not exceed 512")
            } else {
                None
            }
        }
    };
    if let Some(text) = failure {
        error(cx, at, text, INVALID_HEIGHT);
    }
}

/// Part B: a vanilla dimension (its identifier exactly) takes no
/// default biome and no clouds; any other identifier is a custom dimension, which needs a
/// namespace other than `minecraft`. The first failure is the one reported.
fn identifier_rules(dimension: &Dimension, at: &JsonPath, cx: &mut ValidateCx<'_>) {
    let identifier = dimension.description.identifier.as_str();
    let components = &dimension.components;
    let failure = match identifier {
        "minecraft:overworld" | "minecraft:nether" | "minecraft:the_end" => {
            if components.default_biome.is_some() {
                Some((
                    "minecraft:default_biome",
                    "Cannot change the default value of vanilla dimensions biomes!",
                ))
            } else if components.clouds.is_some() {
                Some((
                    "minecraft:clouds",
                    "Cannot change the default height of vanilla dimensions clouds!",
                ))
            } else {
                None
            }
        }
        _ if is_vanilla_namespace(identifier) => Some((
            "identifier",
            "Custom dimensions cannot have the minecraft namespace!",
        )),
        // Any `:`, the first byte included, is a namespace here.
        _ if !identifier.contains(':') => {
            Some(("identifier", "Custom dimensions need to have a namespace!"))
        }
        _ => None,
    };
    if let Some((member, text)) = failure {
        let path = if member == "identifier" {
            child(at, &["description", "identifier"])
        } else {
            child(at, &["components", member])
        };
        error(cx, path, text, INVALID_CUSTOM);
    }
}

/// Whether the identifier is in the `minecraft` namespace: validating it lower-cased succeeds
/// and its namespace is `minecraft`. Validation succeeds for exactly one `:` between two
/// non-empty parts that hold no space, tab, line feed or carriage return; otherwise the
/// identifier is in no namespace, and so not in the vanilla one.
fn is_vanilla_namespace(identifier: &str) -> bool {
    let lower = identifier.to_ascii_lowercase();
    let Some((namespace, name)) = lower.split_once(':') else {
        return false;
    };
    let chunk = |part: &str| !part.is_empty() && !part.contains([' ', '\t', '\n', '\r']);
    namespace == VANILLA_NAMESPACE && chunk(name) && !name.contains(':')
}

/// A validator failure: `InvalidValue` (error) with the content-log text, then the validation
/// error text.
fn error(
    cx: &mut ValidateCx<'_>,
    at: JsonPath,
    content_log: &'static str,
    validation: &'static str,
) {
    cx.diagnostics.push(
        Diagnostic::error(DiagnosticCode::InvalidValue {
            text: Some(VanillaText::Formatted {
                template: "{}\n{}",
                args: vec![content_log.to_owned(), validation.to_owned()],
            }),
        })
        .at(at)
        .phase(Phase::Validate),
    );
}
