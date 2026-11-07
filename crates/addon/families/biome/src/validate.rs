//! The biome family's rules on a loaded document: the identifier, the member and component
//! constraints the struct of the document's version binds, and the checks vanilla runs before
//! it registers a biome. An error rejects the document; the biome is not registered.
//! References to other biomes and to blocks are the link pass's.

use indexmap::IndexSet;

use super::codes;
use super::component::{
    Adjustment, BiomeComponent, BiomeReplacement, Builder, CategoryOrWeight, Climate,
    CreatureSpawnProbability, MultinoiseGenerationRules, NameOrWeight, NoiseBlockSpecifier,
    NoiseGradientSurface, OverworldGenerationRules, OverworldHeight, ReplaceBiomes,
    SubsurfaceBuilder, SurfaceBuilder, SurfaceMaterialAdjustments, WeightedBiome, WeightedBiomes,
    child,
};
use super::read::V1_26_0;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, JsonPath, Phase, Severity};
use bedrock_addon_core::document::ComponentMap;
use bedrock_addon_core::id::BiomeId;
use bedrock_addon_core::shared::RangeForm;
use bedrock_addon_core::versioning::ValidateCx;
use bedrock_addon_shared_shapes::BlockSpecifier;

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

/// An `InvalidValue` without text: a member constraint, whose message is the game's generic
/// one rather than a fixed sentence.
fn constraint() -> Code {
    Code::from(&core_codes::INVALID_VALUE)
}

/// Validates the identifier and the components of a biome (stable or beta).
pub(crate) fn validate(
    identifier: &BiomeId,
    components: &mut ComponentMap<BiomeComponent>,
    cx: &mut ValidateCx<'_>,
) {
    let base_game = cx.load.is_base_game_pack;
    let original = cx.original_version;
    check_identifier(identifier, cx);
    for entry in components.iter() {
        let bedrock_addon_core::document::ComponentRef::Typed { name, value, .. } = entry else {
            continue;
        };
        let base = ["components", name];
        check_blocks(value, &base, cx);
        match value {
            BiomeComponent::Climate(c) => climate(c, &base, cx),
            BiomeComponent::CreatureSpawnProbability(c) => spawn_probability(c, &base, cx),
            BiomeComponent::OverworldHeight(c) => overworld_height(c, &base, cx),
            BiomeComponent::OverworldGenerationRules(c) => {
                overworld_rules(c, &base, cx);
            }
            BiomeComponent::ReplaceBiomes(c) => {
                replace_biomes(c, original < V1_26_0, &base, cx);
            }
            BiomeComponent::SurfaceBuilder(SurfaceBuilder { builder, .. })
            | BiomeComponent::SubsurfaceBuilder(SubsurfaceBuilder { builder, .. }) => {
                surface_builder(builder, base_game, &base, cx);
            }
            BiomeComponent::SurfaceMaterialAdjustments(c) => {
                adjustments(c, &base, cx);
            }
            BiomeComponent::MultinoiseGenerationRules(MultinoiseGenerationRules { .. }) => {
                multinoise(identifier, cx);
            }
            _ => {}
        }
    }
    #[cfg(feature = "molang")]
    molang_failures(components, &cx_failures(&cx.molang_failures), cx.load, cx);
    nether(components, cx);
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

fn check_identifier(identifier: &BiomeId, cx: &mut ValidateCx<'_>) {
    if !identifier_matches(identifier.as_str()) {
        cx.diagnostics.push(
            Diagnostic::of(&codes::IDENTIFIER)
                .at(child(&cx.at, &["description", "identifier"]))
                .phase(Phase::Validate),
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
fn check_blocks(component: &BiomeComponent, base: &[&str], cx: &mut ValidateCx<'_>) {
    for (path, block) in blocks(component) {
        let mut at = child(&cx.at, base);
        at.append(&path);
        cx.diagnostics.extend(block.check(&at, Severity::Warning));
    }
}

// ---- component checks and member constraints -----------------------------------------------

/// `minecraft:climate`: the snow cover's minimum must not exceed its maximum.
fn climate(c: &Climate, base: &[&str], cx: &mut ValidateCx<'_>) {
    if let Some([min, max]) = c.snow_accumulation
        && min > max
    {
        cx.diagnostics.push(
            Diagnostic::of(&codes::SNOW_ACCUMULATION)
                .at(child(&cx.at, &[base, &["snow_accumulation"]].concat()))
                .phase(Phase::Validate),
        );
    }
}

/// `minecraft:creature_spawn_probability`: `[0.0, 0.75]`.
fn spawn_probability(c: &CreatureSpawnProbability, base: &[&str], cx: &mut ValidateCx<'_>) {
    if let Some(p) = c.probability
        && !(0.0..=0.75).contains(&p)
    {
        cx.diagnostics.push(
            Diagnostic::error(constraint())
                .at(child(&cx.at, &[base, &["probability"]].concat()))
                .phase(Phase::Validate),
        );
    }
}

/// `minecraft:overworld_height`: a preset or `[depth, scale]`, not both.
fn overworld_height(c: &OverworldHeight, base: &[&str], cx: &mut ValidateCx<'_>) {
    if c.noise_type.is_some() && c.noise_params.is_some() {
        cx.diagnostics.push(
            Diagnostic::of(&codes::BOTH_NOISE)
                .at(child(&cx.at, base))
                .phase(Phase::Validate),
        );
    }
}

/// `minecraft:overworld_generation_rules`: each two-element entry is a name and a weight (a
/// category and a weight for the climates). The names are the link pass's.
fn overworld_rules(c: &OverworldGenerationRules, base: &[&str], cx: &mut ValidateCx<'_>) {
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
                cx.diagnostics.push(
                    Diagnostic::of(&codes::WEIGHTED_BIOME)
                        .at(child(&cx.at, &[base, &[key, &index.to_string()]].concat()))
                        .phase(Phase::Validate),
                );
            }
        }
    }
    for (index, climate) in c.generate_for_climates.iter().flatten().enumerate() {
        if !matches!(
            climate.0,
            [CategoryOrWeight::Category(_), CategoryOrWeight::Weight(_)]
        ) {
            cx.diagnostics.push(
                Diagnostic::of(&codes::WEIGHTED_CLIMATE)
                    .at(child(
                        &cx.at,
                        &[base, &["generate_for_climates", &index.to_string()]].concat(),
                    ))
                    .phase(Phase::Validate),
            );
        }
    }
}

/// `minecraft:replace_biomes`: the component constraint of the document's struct, the member
/// constraints, and the multiple-dimension warning. The targets are the link pass's. A
/// document older than 1.26.0 gets the older texts and does not accept the Nether.
fn replace_biomes(c: &ReplaceBiomes, before_1_26_0: bool, base: &[&str], cx: &mut ValidateCx<'_>) {
    let replacements = [base, &["replacements"]].concat();
    if c.replacements.is_empty() {
        let empty = if before_1_26_0 {
            &codes::EMPTY_REPLACEMENTS_1_21_80
        } else {
            &codes::EMPTY_REPLACEMENTS_1_26_0
        };
        cx.diagnostics.push(
            Diagnostic::of(empty)
                .at(child(&cx.at, &replacements))
                .phase(Phase::Validate),
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
        let dimension = dimension.as_str();
        dimensions.insert(dimension);
        let accepted = dimension == OVERWORLD || (!before_1_26_0 && dimension == NETHER);
        if !accepted {
            let code = if before_1_26_0 {
                codes::DIMENSION_1_21_80.with([OVERWORLD, dimension])
            } else {
                codes::DIMENSION_1_26_0.with([OVERWORLD, NETHER, dimension])
            };
            cx.diagnostics.push(
                Diagnostic::of(code)
                    .at(child(&cx.at, &at("dimension")))
                    .phase(Phase::Validate),
            );
        }
        for target in targets {
            if !pairs.insert((dimension, target.as_str())) {
                cx.diagnostics.push(
                    Diagnostic::of(codes::DUPLICATE_PAIR.with([dimension, target.as_str()]))
                        .at(child(&cx.at, &at("targets")))
                        .phase(Phase::Validate),
                );
            }
        }
        if targets.is_empty() {
            cx.diagnostics.push(
                Diagnostic::error(constraint())
                    .at(child(&cx.at, &at("targets")))
                    .phase(Phase::Validate),
            );
        }
        if !(*amount > 0.0 && *amount <= 1.0) {
            cx.diagnostics.push(
                Diagnostic::error(constraint())
                    .at(child(&cx.at, &at("amount")))
                    .phase(Phase::Validate),
            );
        }
        if !(*noise_frequency_scale > 0.0 && *noise_frequency_scale <= 100.0) {
            cx.diagnostics.push(
                Diagnostic::error(constraint())
                    .at(child(&cx.at, &at("noise_frequency_scale")))
                    .phase(Phase::Validate),
            );
        }
    }
    // Replacements in more than one dimension are a level-2 line in the game's content log:
    // validation continues.
    if dimensions.len() > 1 {
        cx.diagnostics.push(
            Diagnostic::of(&codes::MULTIPLE_DIMENSIONS)
                .at(child(&cx.at, &replacements))
                .phase(Phase::Validate),
        );
    }
}

/// A builder: the depth bounds, the capped lists and the noise gradient's constraints.
fn surface_builder(builder: &Builder, base_game: bool, base: &[&str], cx: &mut ValidateCx<'_>) {
    let at = |member: &'static str| [base, &["builder", member]].concat();
    let depth = |value: i32| (0..=127).contains(&value);
    match builder {
        Builder::Overworld(s) | Builder::FrozenOcean(s) => {
            if !depth(s.sea_floor_depth) {
                cx.diagnostics.push(
                    Diagnostic::error(constraint())
                        .at(child(&cx.at, &at("sea_floor_depth")))
                        .phase(Phase::Validate),
                );
            }
        }
        Builder::Swamp(s) => {
            if !depth(s.sea_floor_depth) {
                cx.diagnostics.push(
                    Diagnostic::error(constraint())
                        .at(child(&cx.at, &at("sea_floor_depth")))
                        .phase(Phase::Validate),
                );
            }
            // The puddle depth has the same closed bounds, 0 to 127.
            if !depth(s.max_puddle_depth_below_sea_level) {
                cx.diagnostics.push(
                    Diagnostic::error(constraint())
                        .at(child(&cx.at, &at("max_puddle_depth_below_sea_level")))
                        .phase(Phase::Validate),
                );
            }
        }
        Builder::Mesa(s) => {
            if !depth(s.sea_floor_depth) {
                cx.diagnostics.push(
                    Diagnostic::error(constraint())
                        .at(child(&cx.at, &at("sea_floor_depth")))
                        .phase(Phase::Validate),
                );
            }
        }
        Builder::Capped(s) => {
            if s.floor_materials.is_empty() {
                cx.diagnostics.push(
                    Diagnostic::error(constraint())
                        .at(child(&cx.at, &at("floor_materials")))
                        .phase(Phase::Validate),
                );
            }
            if s.ceiling_materials.is_empty() {
                cx.diagnostics.push(
                    Diagnostic::error(constraint())
                        .at(child(&cx.at, &at("ceiling_materials")))
                        .phase(Phase::Validate),
                );
            }
        }
        Builder::TheEnd(_) => {}
        Builder::NoiseGradient(s) => {
            noise_gradient(s, base_game, &[base, &["builder"]].concat(), cx);
        }
    }
}

/// The 1.26.30 noise gradient: amplitudes 1 to 100, at least one specifier, each specifier's
/// `noise` constraint and range form, then the overlap check.
fn noise_gradient(
    s: &NoiseGradientSurface,
    base_game: bool,
    base: &[&str],
    cx: &mut ValidateCx<'_>,
) {
    let amplitudes = s.noise_descriptor.amplitudes.len();
    // At least 1 and at most 100 amplitudes.
    if !(1..=100).contains(&amplitudes) {
        cx.diagnostics.push(
            Diagnostic::error(constraint())
                .at(child(
                    &cx.at,
                    &[base, &["noise_descriptor", "amplitudes"]].concat(),
                ))
                .phase(Phase::Validate),
        );
    }
    // At least one specifier.
    if s.noise_block_specifiers.is_empty() {
        cx.diagnostics.push(
            Diagnostic::error(constraint())
                .at(child(&cx.at, &[base, &["noise_block_specifiers"]].concat()))
                .phase(Phase::Validate),
        );
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
            && let Some(code) = noise_constraint(noise, base_game)
        {
            cx.diagnostics.push(
                Diagnostic::error(code)
                    .at(child(&cx.at, &at("noise")))
                    .phase(Phase::Validate),
            );
            failed = true;
        }
        if let Some(range) = &specifier.range {
            // The game reads this range as an object `{min, max}` only; any other form
            // ("expected an object") counts as an error under the strict mask every biome
            // slice uses, so the biome is refused.
            if range.form != RangeForm::MinMax {
                cx.diagnostics.push(
                    Diagnostic::of(&core_codes::INVALID_TYPE)
                        .at(child(&cx.at, &at("range")))
                        .phase(Phase::Validate),
                );
                failed = true;
            }
            cx.diagnostics
                .extend(range.check(&child(&cx.at, &at("range"))));
        }
    }
    if failed {
        return;
    }
    if overlapping(&s.noise_block_specifiers) {
        let at = [base, &["noise_block_specifiers"]].concat();
        cx.diagnostics.push(
            Diagnostic::of(&codes::OVERLAP)
                .at(child(&cx.at, &at))
                .phase(Phase::Validate),
        );
        cx.diagnostics.push(
            Diagnostic::of(&codes::SURFACE_BUILDER_UNRESOLVED)
                .at(child(&cx.at, &base[..base.len() - 1]))
                .phase(Phase::Validate),
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
fn noise_constraint(noise: &str, base_game: bool) -> Option<Code> {
    // `^(?:{P}.)+:(?:.)+$`: something, a colon, something.
    let namespaced = noise
        .char_indices()
        .any(|(i, c)| c == ':' && i > 0 && i + 1 < noise.len());
    let in_minecraft = noise
        .get(..10)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("minecraft:"));
    if !namespaced {
        return Some(Code::from(&codes::NAMESPACE_MISSING));
    }
    if !base_game && in_minecraft {
        return Some(codes::NAMESPACE_INVALID.with(["minecraft"]));
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
fn adjustments(c: &SurfaceMaterialAdjustments, base: &[&str], cx: &mut ValidateCx<'_>) {
    for (index, adjustment) in c.adjustments.iter().flatten().enumerate() {
        let i = index.to_string();
        let Adjustment {
            noise_range,
            height_range,
            ..
        } = adjustment;
        if let Some([low, high]) = noise_range
            && high <= low
        {
            cx.diagnostics.push(
                Diagnostic::of(&codes::NOISE_RANGE_BOUNDS)
                    .at(child(
                        &cx.at,
                        &[base, &["adjustments", &i, "noise_range"]].concat(),
                    ))
                    .phase(Phase::Validate),
            );
        }
        if let Some([low, high]) = height_range
            && let (Some(low), Some(high)) = (low.constant_value(), high.constant_value())
            && high <= low
        {
            cx.diagnostics.push(
                Diagnostic::of(&codes::HEIGHT_RANGE_BOUNDS)
                    .at(child(
                        &cx.at,
                        &[base, &["adjustments", &i, "height_range"]].concat(),
                    ))
                    .phase(Phase::Validate),
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
    cx: &mut ValidateCx<'_>,
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
            let full = child(&cx.at, &path);
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
                cx.diagnostics.push(
                    Diagnostic::of(codes::EXPRESSION.with([source.as_str()]))
                        .at(child(&cx.at, &path))
                        .phase(Phase::Validate),
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
            cx.diagnostics.push(
                Diagnostic::of(&codes::HEIGHT_RANGE_BOUNDS)
                    .at(child(
                        &cx.at,
                        &[&base[..], &["adjustments", &i, "height_range"]].concat(),
                    ))
                    .phase(Phase::Validate),
            );
        }
    }
}

/// `minecraft:multinoise_generation_rules` is applied only to `minecraft` biomes. For any
/// other biome the game logs a level-3 line and skips the component while the biome is still
/// registered: a kept error, with the component kept in the document.
fn multinoise(identifier: &BiomeId, cx: &mut ValidateCx<'_>) {
    if identifier.namespace() != Some("minecraft") {
        cx.diagnostics.push_kept_error(
            Diagnostic::of(
                codes::PRE_CAVES
                    .with([identifier.as_str(), "minecraft:multinoise_generation_rules"]),
            )
            .at(child(
                &cx.at,
                &["components", "minecraft:multinoise_generation_rules"],
            ))
            .phase(Phase::Validate),
        );
    }
}

/// A biome that replaces Nether biomes needs a builder the Nether takes; a subsurface builder
/// has no effect there. Without a surface builder the biome passes (the crate's own choice).
fn nether(components: &ComponentMap<BiomeComponent>, cx: &mut ValidateCx<'_>) {
    let replaces_nether = components
        .get::<ReplaceBiomes>()
        .is_some_and(|r| r.replacements.iter().any(|x| x.dimension == NETHER));
    if !replaces_nether {
        return;
    }
    if let Some(surface) = components.get::<SurfaceBuilder>()
        && !NETHER_TYPES.contains(&surface.builder.type_name())
    {
        cx.diagnostics.push(
            Diagnostic::of(&codes::NETHER_BUILDERS)
                .at(child(
                    &cx.at,
                    &["components", "minecraft:surface_builder", "builder", "type"],
                ))
                .phase(Phase::Validate),
        );
    }
    if components.contains::<SubsurfaceBuilder>() {
        // The game logs this at level 2, so it is a warning.
        cx.diagnostics.push(
            Diagnostic::of(codes::NO_EFFECT_IN_NETHER.with(["minecraft:subsurface_builder"]))
                .at(child(
                    &cx.at,
                    &["components", "minecraft:subsurface_builder"],
                ))
                .phase(Phase::Validate),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{identifier_matches, noise_constraint};
    use bedrock_addon_core::diagnostic::Code;
    use bedrock_addon_core::diagnostic::codes as core_codes;

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
        let namespace = |c: Option<Code>| {
            c.is_some_and(|c| {
                matches!(
                    c.id(),
                    "biome/namespace_invalid" | "biome/namespace_missing"
                )
            })
        };
        assert_eq!(noise_constraint("test:grad", false), None);
        assert!(namespace(noise_constraint("minecraft:grad", false)));
        assert!(namespace(noise_constraint("MineCraft:grad", false)));
        assert!(namespace(noise_constraint("grad", false)));
        assert_eq!(noise_constraint("minecraft:grad", true), None);
        assert!(
            noise_constraint("test:grad", true).is_some_and(|c| c.is(&core_codes::INVALID_VALUE))
        );
        assert!(noise_constraint("a b:c", false).is_some_and(|c| c.is(&core_codes::INVALID_VALUE)));
    }
}
