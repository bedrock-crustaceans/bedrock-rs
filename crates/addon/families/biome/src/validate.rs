//! The biome family's rules on a loaded document: the identifier, the member and component
//! constraints the struct of the document's version binds, and the checks vanilla runs before
//! it registers a biome. An error rejects the document; the biome is not registered.
//! References to other biomes and to blocks are the link pass's.

use indexmap::IndexSet;

use super::component::{
    Adjustment, BiomeComponent, BiomeReplacement, Builder, CategoryOrWeight, Climate,
    CreatureSpawnProbability, MultinoiseGenerationRules, NameOrWeight, NoiseBlockSpecifier,
    NoiseGradientSurface, OverworldGenerationRules, OverworldHeight, ReplaceBiomes,
    SubsurfaceBuilder, SurfaceBuilder, SurfaceMaterialAdjustments, WeightedBiome, WeightedBiomes,
    child,
};
use super::parse::V1_26_0;
use super::texts::{
    BOTH_NOISE, DIMENSION_1_21_80, DIMENSION_1_26_0, DUPLICATE_PAIR, EMPTY_REPLACEMENTS_1_21_80,
    EMPTY_REPLACEMENTS_1_26_0, HEIGHT_RANGE_BOUNDS, IDENTIFIER, MULTIPLE_DIMENSIONS,
    NAMESPACE_INVALID, NAMESPACE_MISSING, NETHER_BUILDERS, NO_EFFECT_IN_NETHER, NOISE_RANGE_BOUNDS,
    OVERLAP, PRE_CAVES, SNOW_ACCUMULATION, SURFACE_BUILDER_UNRESOLVED, WEIGHTED_BIOME,
    WEIGHTED_CLIMATE, fixed, text,
};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::document::ComponentMap;
use bedrock_addon_core::id::BiomeId;
use bedrock_addon_core::shared::{BlockSpecifier, RangeForm};
use bedrock_addon_core::versioning::ValidateCx;

#[cfg(feature = "molang")]
use super::texts::EXPRESSION;

/// The overworld.
const OVERWORLD: &str = "minecraft:overworld";
/// The Nether.
const NETHER: &str = "minecraft:nether";
/// The builder types the Nether takes: `overworld`, `frozen_ocean`, `capped` and `the_end`.
const NETHER_TYPES: [&str; 4] = [
    "minecraft:overworld",
    "minecraft:frozen_ocean",
    "minecraft:capped",
    "minecraft:the_end",
];

/// Where the findings go: the payload's path and the diagnostics.
struct Report<'a> {
    at: &'a JsonPath,
    d: &'a mut Diagnostics,
}

impl Report<'_> {
    fn push(&mut self, code: DiagnosticCode, severity: Severity, path: &[&str]) {
        self.d.push(Diagnostic::new(
            code,
            severity,
            Phase::Validate,
            Location::new(None, None, child(self.at, path), None),
        ));
    }

    fn error(&mut self, code: DiagnosticCode, path: &[&str]) {
        self.push(code, Severity::Error, path);
    }

    /// A warning: the game's level-2 content-log line, after which validation continues.
    fn warn(&mut self, code: DiagnosticCode, path: &[&str]) {
        self.push(code, Severity::Warning, path);
    }

    /// A kept error: the game's level-3 line after which the biome still registers.
    fn kept(&mut self, code: DiagnosticCode, path: &[&str]) {
        self.d.push_kept_error(Diagnostic::new(
            code,
            Severity::Error,
            Phase::Validate,
            Location::new(None, None, child(self.at, path), None),
        ));
    }
}

/// An `InvalidValue` without text: a member constraint, whose message is the game's generic
/// one rather than a fixed sentence.
fn constraint() -> DiagnosticCode {
    DiagnosticCode::InvalidValue { text: None }
}

/// Validates the identifier and the components of a biome (stable or beta).
pub(crate) fn validate(
    identifier: &BiomeId,
    components: &mut ComponentMap<BiomeComponent>,
    cx: &mut ValidateCx<'_>,
) {
    let at = cx.at.clone();
    let base_game = cx.load.is_base_game_pack;
    let original = cx.original_version;
    let mut report = Report {
        at: &at,
        d: &mut *cx.diagnostics,
    };
    check_identifier(identifier, &mut report);
    for entry in components.iter() {
        let bedrock_addon_core::document::ComponentRef::Typed { name, value, .. } = entry else {
            continue;
        };
        let base = ["components", name];
        check_blocks(value, &base, &mut report);
        match value {
            BiomeComponent::Climate(c) => climate(c, &base, &mut report),
            BiomeComponent::CreatureSpawnProbability(c) => spawn_probability(c, &base, &mut report),
            BiomeComponent::OverworldHeight(c) => overworld_height(c, &base, &mut report),
            BiomeComponent::OverworldGenerationRules(c) => {
                overworld_rules(c, &base, &mut report);
            }
            BiomeComponent::ReplaceBiomes(c) => {
                replace_biomes(c, original < V1_26_0, &base, &mut report);
            }
            BiomeComponent::SurfaceBuilder(SurfaceBuilder { builder, .. })
            | BiomeComponent::SubsurfaceBuilder(SubsurfaceBuilder { builder, .. }) => {
                surface_builder(builder, base_game, &base, &mut report);
            }
            BiomeComponent::SurfaceMaterialAdjustments(c) => {
                adjustments(c, &base, &mut report);
            }
            BiomeComponent::MultinoiseGenerationRules(MultinoiseGenerationRules { .. }) => {
                multinoise(identifier, &mut report);
            }
            _ => {}
        }
    }
    #[cfg(feature = "molang")]
    molang_failures(
        components,
        &cx_failures(&cx.molang_failures),
        cx.load,
        &mut report,
    );
    nether(components, &mut report);
}

/// The fields whose height-range expression failed to compile.
#[cfg(feature = "molang")]
fn cx_failures(failures: &[bedrock_addon_core::molang::FieldOutcome]) -> Vec<JsonPath> {
    use bedrock_addon_core::molang::MolangOutcome;
    failures
        .iter()
        .filter(|f| matches!(f.outcome, MolangOutcome::Failed(_)))
        .map(|f| f.path.clone())
        .collect()
}

// ---- the identifier ------------------------------------------------------------------------

/// `^[a-z0-9._%+-]+:[a-z0-9._%+-]+$`, the identifier's regex.
fn identifier_matches(identifier: &str) -> bool {
    let part = |s: &str| {
        !s.is_empty()
            && s.bytes().all(|b| {
                b.is_ascii_lowercase()
                    || b.is_ascii_digit()
                    || matches!(b, b'.' | b'_' | b'%' | b'+' | b'-')
            })
    };
    identifier
        .split_once(':')
        .is_some_and(|(namespace, name)| part(namespace) && part(name))
}

fn check_identifier(identifier: &BiomeId, report: &mut Report<'_>) {
    if !identifier_matches(identifier.as_str()) {
        report.error(
            DiagnosticCode::NamespaceRule {
                text: fixed(IDENTIFIER),
            },
            &["description", "identifier"],
        );
    }
}

// ---- block specifiers: unknown members -----------------------------------------------------

/// Every block specifier of a component, with its path below the component.
pub fn blocks(component: &BiomeComponent) -> Vec<(JsonPath, &BlockSpecifier)> {
    fn add<'b>(
        out: &mut Vec<(JsonPath, &'b BlockSpecifier)>,
        path: &[&str],
        block: &'b BlockSpecifier,
    ) {
        out.push((child(&JsonPath::new(), path), block));
    }
    let mut out = Vec::new();
    match component {
        BiomeComponent::MountainParameters(c) => {
            if let Some(material) = c
                .steep_material_adjustment
                .as_ref()
                .and_then(|s| s.material.as_ref())
            {
                add(
                    &mut out,
                    &["steep_material_adjustment", "material"],
                    material,
                );
            }
        }
        BiomeComponent::SurfaceBuilder(SurfaceBuilder { builder, .. })
        | BiomeComponent::SubsurfaceBuilder(SubsurfaceBuilder { builder, .. }) => {
            builder_blocks(builder, &mut out);
        }
        BiomeComponent::SurfaceMaterialAdjustments(c) => {
            for (index, adjustment) in c.adjustments.iter().flatten().enumerate() {
                let i = index.to_string();
                let m = &adjustment.materials;
                for (key, block) in [
                    ("top_material", &m.top_material),
                    ("mid_material", &m.mid_material),
                    ("sea_floor_material", &m.sea_floor_material),
                    ("foundation_material", &m.foundation_material),
                    ("sea_material", &m.sea_material),
                ] {
                    if let Some(block) = block {
                        add(&mut out, &["adjustments", &i, "materials", key], block);
                    }
                }
            }
        }
        _ => {}
    }
    out
}

/// The block specifiers of a builder, below `builder`.
fn builder_blocks<'a>(builder: &'a Builder, out: &mut Vec<(JsonPath, &'a BlockSpecifier)>) {
    let mut add = |path: &[&str], block: &'a BlockSpecifier| {
        out.push((
            child(&JsonPath::new(), &[&["builder"], path].concat()),
            block,
        ));
    };
    let layered = |s: [&'a BlockSpecifier; 5]| {
        [
            ("sea_floor_material", s[0]),
            ("foundation_material", s[1]),
            ("mid_material", s[2]),
            ("top_material", s[3]),
            ("sea_material", s[4]),
        ]
    };
    match builder {
        Builder::Overworld(s) | Builder::FrozenOcean(s) => {
            for (key, block) in layered([
                &s.sea_floor_material,
                &s.foundation_material,
                &s.mid_material,
                &s.top_material,
                &s.sea_material,
            ]) {
                add(&[key], block);
            }
        }
        Builder::Swamp(s) => {
            for (key, block) in layered([
                &s.sea_floor_material,
                &s.foundation_material,
                &s.mid_material,
                &s.top_material,
                &s.sea_material,
            ]) {
                add(&[key], block);
            }
        }
        Builder::Mesa(s) => {
            for (key, block) in layered([
                &s.sea_floor_material,
                &s.foundation_material,
                &s.mid_material,
                &s.top_material,
                &s.sea_material,
            ]) {
                add(&[key], block);
            }
            add(&["clay_material"], &s.clay_material);
            add(&["hard_clay_material"], &s.hard_clay_material);
        }
        Builder::Capped(s) => {
            for (index, block) in s.floor_materials.iter().enumerate() {
                add(&["floor_materials", &index.to_string()], block);
            }
            for (index, block) in s.ceiling_materials.iter().enumerate() {
                add(&["ceiling_materials", &index.to_string()], block);
            }
            add(&["sea_material"], &s.sea_material);
            add(&["foundation_material"], &s.foundation_material);
            if let Some(beach) = &s.beach_material {
                add(&["beach_material"], beach);
            }
        }
        Builder::TheEnd(_) => {}
        Builder::NoiseGradient(s) => {
            for (index, block) in s.non_replaceable_blocks.iter().flatten().enumerate() {
                add(&["non_replaceable_blocks", &index.to_string()], block);
            }
            for (index, specifier) in s.noise_block_specifiers.iter().enumerate() {
                add(
                    &["noise_block_specifiers", &index.to_string(), "block"],
                    &specifier.block,
                );
            }
        }
    }
}

/// The unknown members of each block specifier (`BlockSpecifier::check`).
fn check_blocks(component: &BiomeComponent, base: &[&str], report: &mut Report<'_>) {
    for (path, block) in blocks(component) {
        let mut at = child(report.at, base);
        at.append(&path);
        report.d.extend(block.check(&at));
    }
}

// ---- component checks and member constraints -----------------------------------------------

/// `minecraft:climate`: the snow cover's minimum must not exceed its maximum.
fn climate(c: &Climate, base: &[&str], report: &mut Report<'_>) {
    if let Some([min, max]) = c.snow_accumulation
        && min.0 > max.0
    {
        report.error(
            DiagnosticCode::InvalidValue {
                text: fixed(SNOW_ACCUMULATION),
            },
            &[base, &["snow_accumulation"]].concat(),
        );
    }
}

/// `minecraft:creature_spawn_probability`: `[0.0, 0.75]`.
fn spawn_probability(c: &CreatureSpawnProbability, base: &[&str], report: &mut Report<'_>) {
    if let Some(p) = c.probability
        && !(0.0..=0.75).contains(&p.0)
    {
        report.error(constraint(), &[base, &["probability"]].concat());
    }
}

/// `minecraft:overworld_height`: a preset or `[depth, scale]`, not both.
fn overworld_height(c: &OverworldHeight, base: &[&str], report: &mut Report<'_>) {
    if c.noise_type.is_some() && c.noise_params.is_some() {
        report.error(
            DiagnosticCode::InvalidValue {
                text: fixed(BOTH_NOISE),
            },
            base,
        );
    }
}

/// `minecraft:overworld_generation_rules`: each two-element entry is a name and a weight (a
/// category and a weight for the climates). The names are the link pass's.
fn overworld_rules(c: &OverworldGenerationRules, base: &[&str], report: &mut Report<'_>) {
    for (key, transformation) in [
        ("hills_transformation", &c.hills_transformation),
        ("mutate_transformation", &c.mutate_transformation),
        ("river_transformation", &c.river_transformation),
        ("shore_transformation", &c.shore_transformation),
    ] {
        let Some(WeightedBiomes::List(entries)) = transformation else {
            continue;
        };
        for (index, entry) in entries.iter().enumerate() {
            let ordered = matches!(
                entry,
                WeightedBiome::Name(_)
                    | WeightedBiome::Pair([NameOrWeight::Name(_), NameOrWeight::Weight(_)])
            );
            if !ordered {
                report.error(
                    DiagnosticCode::InvalidValue {
                        text: fixed(WEIGHTED_BIOME),
                    },
                    &[base, &[key, &index.to_string()]].concat(),
                );
            }
        }
    }
    for (index, climate) in c.generate_for_climates.iter().flatten().enumerate() {
        if !matches!(
            climate.0,
            [CategoryOrWeight::Category(_), CategoryOrWeight::Weight(_)]
        ) {
            report.error(
                DiagnosticCode::InvalidValue {
                    text: fixed(WEIGHTED_CLIMATE),
                },
                &[base, &["generate_for_climates", &index.to_string()]].concat(),
            );
        }
    }
}

/// `minecraft:replace_biomes`: the component constraint of the document's struct, the member
/// constraints, and the multiple-dimension warning. The targets are the link pass's. A
/// document older than 1.26.0 gets the older texts and does not accept the Nether.
fn replace_biomes(c: &ReplaceBiomes, before_1_26_0: bool, base: &[&str], report: &mut Report<'_>) {
    let replacements = [base, &["replacements"]].concat();
    if c.replacements.is_empty() {
        let empty = if before_1_26_0 {
            EMPTY_REPLACEMENTS_1_21_80
        } else {
            EMPTY_REPLACEMENTS_1_26_0
        };
        report.error(
            DiagnosticCode::InvalidValue { text: fixed(empty) },
            &replacements,
        );
        return;
    }
    let mut pairs = IndexSet::new();
    let mut dimensions = IndexSet::new();
    for (index, replacement) in c.replacements.iter().enumerate() {
        let i = index.to_string();
        let at = |member: &'static str| {
            let mut path: Vec<&str> = replacements.clone();
            path.push(&i);
            path.push(member);
            path
        };
        let BiomeReplacement {
            dimension,
            targets,
            amount,
            noise_frequency_scale,
            ..
        } = replacement;
        let dimension = dimension.0.as_str();
        dimensions.insert(dimension);
        let accepted = dimension == OVERWORLD || (!before_1_26_0 && dimension == NETHER);
        if !accepted {
            let message = if before_1_26_0 {
                text(DIMENSION_1_21_80, &[OVERWORLD, dimension])
            } else {
                text(DIMENSION_1_26_0, &[OVERWORLD, NETHER, dimension])
            };
            report.error(
                DiagnosticCode::InvalidValue { text: message },
                &at("dimension"),
            );
        }
        for target in targets {
            if !pairs.insert((dimension, target.0.as_str())) {
                report.error(
                    DiagnosticCode::InvalidValue {
                        text: text(DUPLICATE_PAIR, &[dimension, &target.0]),
                    },
                    &at("targets"),
                );
            }
        }
        if targets.is_empty() {
            report.error(constraint(), &at("targets"));
        }
        if !(amount.0 > 0.0 && amount.0 <= 1.0) {
            report.error(constraint(), &at("amount"));
        }
        if !(noise_frequency_scale.0 > 0.0 && noise_frequency_scale.0 <= 100.0) {
            report.error(constraint(), &at("noise_frequency_scale"));
        }
    }
    // Replacements in more than one dimension are a level-2 line in the game's content log:
    // validation continues.
    if dimensions.len() > 1 {
        report.warn(
            DiagnosticCode::InvalidValue {
                text: fixed(MULTIPLE_DIMENSIONS),
            },
            &replacements,
        );
    }
}

/// A builder: the depth bounds, the capped lists and the noise gradient's constraints.
fn surface_builder(builder: &Builder, base_game: bool, base: &[&str], report: &mut Report<'_>) {
    let at = |member: &'static str| [base, &["builder", member]].concat();
    let depth = |value: i32| (0..=127).contains(&value);
    match builder {
        Builder::Overworld(s) | Builder::FrozenOcean(s) => {
            if !depth(s.sea_floor_depth.0) {
                report.error(constraint(), &at("sea_floor_depth"));
            }
        }
        Builder::Swamp(s) => {
            if !depth(s.sea_floor_depth.0) {
                report.error(constraint(), &at("sea_floor_depth"));
            }
            // The puddle depth has the same closed bounds, 0 to 127.
            if !depth(s.max_puddle_depth_below_sea_level.0) {
                report.error(constraint(), &at("max_puddle_depth_below_sea_level"));
            }
        }
        Builder::Mesa(s) => {
            if !depth(s.sea_floor_depth.0) {
                report.error(constraint(), &at("sea_floor_depth"));
            }
        }
        Builder::Capped(s) => {
            if s.floor_materials.is_empty() {
                report.error(constraint(), &at("floor_materials"));
            }
            if s.ceiling_materials.is_empty() {
                report.error(constraint(), &at("ceiling_materials"));
            }
        }
        Builder::TheEnd(_) => {}
        Builder::NoiseGradient(s) => {
            noise_gradient(s, base_game, &[base, &["builder"]].concat(), report);
        }
    }
}

/// The 1.26.30 noise gradient: amplitudes 1 to 100, at least one specifier, each specifier's
/// `noise` constraint and range form, then the overlap check.
fn noise_gradient(
    s: &NoiseGradientSurface,
    base_game: bool,
    base: &[&str],
    report: &mut Report<'_>,
) {
    let amplitudes = s.noise_descriptor.amplitudes.len();
    // At least 1 and at most 100 amplitudes.
    if !(1..=100).contains(&amplitudes) {
        report.error(
            constraint(),
            &[base, &["noise_descriptor", "amplitudes"]].concat(),
        );
    }
    // At least one specifier.
    if s.noise_block_specifiers.is_empty() {
        report.error(constraint(), &[base, &["noise_block_specifiers"]].concat());
        return;
    }
    let mut failed = false;
    for (index, specifier) in s.noise_block_specifiers.iter().enumerate() {
        let i = index.to_string();
        let at = |member: &'static str| {
            let mut path: Vec<&str> = base.to_vec();
            path.extend(["noise_block_specifiers", i.as_str(), member]);
            path
        };
        if let Some(noise) = &specifier.noise
            && let Some(code) = noise_constraint(&noise.0, base_game)
        {
            report.error(code, &at("noise"));
            failed = true;
        }
        if let Some(range) = &specifier.range {
            // The game reads this range as an object `{min, max}` only; any other form
            // ("expected an object") counts as an error under the strict mask every biome
            // slice uses, so the biome is refused.
            if range.form != RangeForm::MinMax {
                report.error(DiagnosticCode::InvalidType { text: None }, &at("range"));
                failed = true;
            }
            report
                .d
                .extend(range.check(&child(report.at, &at("range"))));
        }
    }
    if failed {
        return;
    }
    if overlapping(&s.noise_block_specifiers) {
        let at = [base, &["noise_block_specifiers"]].concat();
        report.error(
            DiagnosticCode::InvalidValue {
                text: fixed(OVERLAP),
            },
            &at,
        );
        report.error(
            DiagnosticCode::InvalidValue {
                text: fixed(SURFACE_BUILDER_UNRESOLVED),
            },
            &base[..base.len() - 1],
        );
    }
}

/// Vanilla's pairwise test: two specifiers conflict when each range's maximum is above the
/// other's minimum (strict, so shared endpoints and gaps pass; an inverted range is compared as
/// written, not sorted).
fn overlapping(specifiers: &[NoiseBlockSpecifier]) -> bool {
    let ranges: Vec<(f32, f32)> = specifiers.iter().map(NoiseBlockSpecifier::range).collect();
    ranges.iter().enumerate().any(|(i, a)| {
        ranges
            .iter()
            .enumerate()
            .any(|(j, b)| i != j && a.1 > b.0 && b.1 > a.0)
    })
}

/// The `noise` member's combined constraint: in a base-game pack a `namespace:name` in the
/// `minecraft` namespace, elsewhere a `namespace:name` in any other namespace
/// (case-insensitive); in every pack no whitespace. The first failing part is reported.
fn noise_constraint(noise: &str, base_game: bool) -> Option<DiagnosticCode> {
    // `^(?:{P}.)+:(?:.)+$`: something, a colon, something.
    let namespaced = noise
        .char_indices()
        .any(|(i, c)| c == ':' && i > 0 && i + 1 < noise.len());
    let in_minecraft = noise
        .get(..10)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("minecraft:"));
    if !namespaced {
        return Some(DiagnosticCode::NamespaceRule {
            text: fixed(NAMESPACE_MISSING),
        });
    }
    if !base_game && in_minecraft {
        return Some(DiagnosticCode::NamespaceRule {
            text: text(NAMESPACE_INVALID, &["minecraft"]),
        });
    }
    if base_game && !in_minecraft {
        return Some(constraint());
    }
    if noise.is_empty() || noise.chars().any(char::is_whitespace) {
        return Some(constraint());
    }
    None
}

/// `minecraft:surface_material_adjustments`: constant `noise_range` and `height_range` bounds
/// (the upper above the lower). Block names are the link pass's.
fn adjustments(c: &SurfaceMaterialAdjustments, base: &[&str], report: &mut Report<'_>) {
    for (index, adjustment) in c.adjustments.iter().flatten().enumerate() {
        let i = index.to_string();
        let Adjustment {
            noise_range,
            height_range,
            ..
        } = adjustment;
        if let Some([low, high]) = noise_range
            && high.0 <= low.0
        {
            report.error(
                DiagnosticCode::InvalidValue {
                    text: fixed(NOISE_RANGE_BOUNDS),
                },
                &[base, &["adjustments", &i, "noise_range"]].concat(),
            );
        }
        if let Some([low, high]) = height_range
            && let (Some(low), Some(high)) = (low.constant_value(), high.constant_value())
            && high <= low
        {
            report.error(
                DiagnosticCode::InvalidValue {
                    text: fixed(HEIGHT_RANGE_BOUNDS),
                },
                &[base, &["adjustments", &i, "height_range"]].concat(),
            );
        }
    }
}

/// A `height_range` expression the compile pass rejected fails the component with vanilla's
/// text, after the compiler's own findings; so does one that uses render arrays or resources,
/// which never resolve in world generation. A constant that the compile folds takes part in
/// the bounds check.
#[cfg(feature = "molang")]
fn molang_failures(
    components: &ComponentMap<BiomeComponent>,
    failed: &[JsonPath],
    load: &bedrock_addon_core::load::LoadContext,
    report: &mut Report<'_>,
) {
    use bedrock_addon_core::molang::{CompileFailure, MolangContext, Side};
    use bedrock_addon_core::version::FormatVersionInput;
    let Some(c) = components.get::<SurfaceMaterialAdjustments>() else {
        return;
    };
    let base = ["components", "minecraft:surface_material_adjustments"];
    let check = load.molang.as_ref();
    let written = FormatVersionInput {
        version: None,
        beta_literal: true,
        prerelease: false,
    };
    let molang = MolangContext::new(
        load.min_engine_version.as_ref(),
        load.molang_experiments,
        &written,
        Some(load.molang_version),
    );
    for (index, adjustment) in c.adjustments.iter().flatten().enumerate() {
        let Some(values) = &adjustment.height_range else {
            continue;
        };
        let i = index.to_string();
        let mut folded = [None, None];
        for (slot, value) in values.iter().enumerate() {
            let s = slot.to_string();
            let path: Vec<&str> = [&base[..], &["adjustments", &i, "height_range", &s]].concat();
            let full = child(report.at, &path);
            let Some(source) = value.source() else {
                folded[slot] = value.constant_value();
                continue;
            };
            let rejected = failed.contains(&full);
            let unresolvable = check.is_some_and(|check| {
                let checked = check.compile(
                    source,
                    &super::component::HEIGHT_RANGE,
                    Side::Server,
                    &molang,
                );
                folded[slot] = checked
                    .expr()
                    .and_then(bedrock_addon_core::molang::molang::compile::Expr::as_constant);
                matches!(
                    checked.failure(),
                    Some(CompileFailure::UsesArrays | CompileFailure::UsesResources)
                )
            });
            if rejected || unresolvable {
                report.error(
                    DiagnosticCode::InvalidValue {
                        text: text(EXPRESSION, &[source.as_str()]),
                    },
                    &path,
                );
            }
        }
        // The bounds check of a folded expression; constant values were checked by
        // `adjustments`.
        let both_constant = values.iter().all(|v| v.source().is_none());
        if !both_constant
            && let [Some(low), Some(high)] = folded
            && high <= low
        {
            report.error(
                DiagnosticCode::InvalidValue {
                    text: fixed(HEIGHT_RANGE_BOUNDS),
                },
                &[&base[..], &["adjustments", &i, "height_range"]].concat(),
            );
        }
    }
}

/// `minecraft:multinoise_generation_rules` is applied only to `minecraft` biomes. For any
/// other biome the game logs a level-3 line and skips the component while the biome is still
/// registered: a kept error, with the component kept in the document.
fn multinoise(identifier: &BiomeId, report: &mut Report<'_>) {
    if identifier.namespace() != Some("minecraft") {
        report.kept(
            DiagnosticCode::InvalidValue {
                text: text(
                    PRE_CAVES,
                    &[identifier.as_str(), "minecraft:multinoise_generation_rules"],
                ),
            },
            &["components", "minecraft:multinoise_generation_rules"],
        );
    }
}

/// A biome that replaces Nether biomes needs a builder the Nether takes; a subsurface builder
/// has no effect there. Without a surface builder the biome passes (the crate's own choice).
fn nether(components: &ComponentMap<BiomeComponent>, report: &mut Report<'_>) {
    let replaces_nether = components
        .get::<ReplaceBiomes>()
        .is_some_and(|r| r.replacements.iter().any(|x| x.dimension.0 == NETHER));
    if !replaces_nether {
        return;
    }
    if let Some(surface) = components.get::<SurfaceBuilder>()
        && !NETHER_TYPES.contains(&surface.builder.type_name())
    {
        report.error(
            DiagnosticCode::InvalidValue {
                text: fixed(NETHER_BUILDERS),
            },
            &["components", "minecraft:surface_builder", "builder", "type"],
        );
    }
    if components.contains::<SubsurfaceBuilder>() {
        // The game logs this at level 2, so it is a warning.
        report.warn(
            DiagnosticCode::NoEffect {
                text: text(NO_EFFECT_IN_NETHER, &["minecraft:subsurface_builder"]),
            },
            &["components", "minecraft:subsurface_builder"],
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{identifier_matches, noise_constraint};
    use bedrock_addon_core::diagnostic::DiagnosticCode;

    #[test]
    fn the_identifier_regex_takes_lower_case_namespaced_names() {
        assert!(identifier_matches("demo:my_biome.1-a%b+c"));
        assert!(!identifier_matches("demo:Plains"));
        assert!(!identifier_matches("plains"));
        assert!(!identifier_matches("demo:"));
        assert!(!identifier_matches(":plains"));
        assert!(!identifier_matches("a:b:c"));
    }

    #[test]
    fn the_noise_constraint_follows_the_pack_area() {
        let namespace =
            |c: Option<DiagnosticCode>| matches!(c, Some(DiagnosticCode::NamespaceRule { .. }));
        assert_eq!(noise_constraint("test:grad", false), None);
        assert!(namespace(noise_constraint("minecraft:grad", false)));
        assert!(namespace(noise_constraint("MineCraft:grad", false)));
        assert!(namespace(noise_constraint("grad", false)));
        assert_eq!(noise_constraint("minecraft:grad", true), None);
        assert!(matches!(
            noise_constraint("test:grad", true),
            Some(DiagnosticCode::InvalidValue { .. })
        ));
        assert!(matches!(
            noise_constraint("a b:c", false),
            Some(DiagnosticCode::InvalidValue { .. })
        ));
    }
}
