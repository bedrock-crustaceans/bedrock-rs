//! Constructors of the biome documents.

use super::component::{
    Adjustment, AdjustmentMaterials, BiomeReplacement, Builder, CappedSurface, ColorRgba,
    GrassTint, Humidity, MapTints, MesaSurface, NoiseBlockSpecifier, NoiseDescriptor,
    NoiseGradientSurface, OverworldSurface, ReplaceBiomes, SubsurfaceBuilder, SurfaceBuilder,
    SwampSurface, Tags, TopSlide,
};
use super::{
    Beta, Biome, BoolAttribute, BoolOperation, ColorAttribute, ColorOperation, Description,
    FloatAttribute, FloatOperation,
};
use bedrock_addon_core::document::{ComponentMap, Extra};
use bedrock_addon_core::id::BiomeId;
use bedrock_addon_core::json::{StrictBool, StrictFloat, StrictInt, StrictString};
use bedrock_addon_core::shared::BlockSpecifier;

impl Description {
    /// The description of `identifier`.
    #[must_use]
    pub fn new(identifier: BiomeId) -> Self {
        Self {
            identifier,
            extra: Extra::default(),
        }
    }
}

impl Biome {
    /// A biome named `identifier` without components.
    #[must_use]
    pub fn new(identifier: BiomeId) -> Self {
        Self {
            description: Description::new(identifier),
            components: ComponentMap::new(),
            extra: Extra::default(),
        }
    }
}

impl Beta {
    /// A beta biome named `identifier` without components or environment attributes.
    #[must_use]
    pub fn new(identifier: BiomeId) -> Self {
        Self {
            description: Description::new(identifier),
            components: ComponentMap::new(),
            environment_attributes: None,
            extra: Extra::default(),
        }
    }
}

// ---- components and members with required members ------------------------

impl Humidity {
    /// `minecraft:humidity` with `is_humid`.
    #[must_use]
    pub fn new(is_humid: bool) -> Self {
        Self {
            is_humid: StrictBool(is_humid),
            extra: Extra::default(),
        }
    }
}

impl Tags {
    /// `minecraft:tags` with `tags`, in order.
    pub fn new<S: Into<String>>(tags: impl IntoIterator<Item = S>) -> Self {
        Self {
            tags: tags.into_iter().map(|t| StrictString(t.into())).collect(),
            extra: Extra::default(),
        }
    }
}

impl TopSlide {
    /// `top_slide` with `enabled`.
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled: StrictBool(enabled),
            extra: Extra::default(),
        }
    }
}

impl ReplaceBiomes {
    /// `minecraft:replace_biomes` with `replacements`.
    #[must_use]
    pub fn new(replacements: Vec<BiomeReplacement>) -> Self {
        Self {
            replacements,
            extra: Extra::default(),
        }
    }
}

impl BiomeReplacement {
    /// A replacement of `targets` in `dimension`, by `amount` with noise of
    /// `noise_frequency_scale`.
    pub fn new<S: Into<String>>(
        dimension: impl Into<String>,
        targets: impl IntoIterator<Item = S>,
        amount: f32,
        noise_frequency_scale: f32,
    ) -> Self {
        Self {
            dimension: StrictString(dimension.into()),
            targets: targets
                .into_iter()
                .map(|t| StrictString(t.into()))
                .collect(),
            amount: StrictFloat(amount),
            noise_frequency_scale: StrictFloat(noise_frequency_scale),
            extra: Extra::default(),
        }
    }
}

impl MapTints {
    /// `minecraft:map_tints` with `grass` and no foliage colour.
    #[must_use]
    pub fn new(grass: GrassTint) -> Self {
        Self {
            grass,
            foliage: None,
            extra: Extra::default(),
        }
    }
}

impl SurfaceBuilder {
    /// `minecraft:surface_builder` with `builder`.
    #[must_use]
    pub fn new(builder: Builder) -> Self {
        Self {
            builder,
            extra: Extra::default(),
        }
    }
}

impl SubsurfaceBuilder {
    /// `minecraft:subsurface_builder` with `builder`.
    #[must_use]
    pub fn new(builder: Builder) -> Self {
        Self {
            builder,
            extra: Extra::default(),
        }
    }
}

impl OverworldSurface {
    /// The layered surface: sea floor depth, then the sea floor, foundation, middle, top and
    /// sea blocks.
    #[must_use]
    pub fn new(
        sea_floor_depth: i32,
        sea_floor_material: BlockSpecifier,
        foundation_material: BlockSpecifier,
        mid_material: BlockSpecifier,
        top_material: BlockSpecifier,
        sea_material: BlockSpecifier,
    ) -> Self {
        Self {
            sea_floor_depth: StrictInt(sea_floor_depth),
            sea_floor_material,
            foundation_material,
            mid_material,
            top_material,
            sea_material,
            extra: Extra::default(),
        }
    }
}

impl SwampSurface {
    /// The swamp surface: `surface`'s members and the puddle depth below sea level.
    #[must_use]
    pub fn new(surface: OverworldSurface, max_puddle_depth_below_sea_level: i32) -> Self {
        Self {
            sea_floor_depth: surface.sea_floor_depth,
            sea_floor_material: surface.sea_floor_material,
            foundation_material: surface.foundation_material,
            mid_material: surface.mid_material,
            top_material: surface.top_material,
            sea_material: surface.sea_material,
            max_puddle_depth_below_sea_level: StrictInt(max_puddle_depth_below_sea_level),
            extra: surface.extra,
        }
    }
}

impl CappedSurface {
    /// The capped surface: floor and ceiling blocks, the sea and foundation blocks, no beach.
    #[must_use]
    pub fn new(
        floor_materials: Vec<BlockSpecifier>,
        ceiling_materials: Vec<BlockSpecifier>,
        sea_material: BlockSpecifier,
        foundation_material: BlockSpecifier,
    ) -> Self {
        Self {
            floor_materials,
            ceiling_materials,
            sea_material,
            foundation_material,
            beach_material: None,
            extra: Extra::default(),
        }
    }
}

impl MesaSurface {
    /// The mesa surface: `surface`'s members, the clay and hardened clay, and whether bryce
    /// pillars and a forest generate.
    #[must_use]
    pub fn new(
        surface: OverworldSurface,
        clay_material: BlockSpecifier,
        hard_clay_material: BlockSpecifier,
        bryce_pillars: bool,
        has_forest: bool,
    ) -> Self {
        Self {
            sea_floor_depth: surface.sea_floor_depth,
            sea_floor_material: surface.sea_floor_material,
            foundation_material: surface.foundation_material,
            mid_material: surface.mid_material,
            top_material: surface.top_material,
            sea_material: surface.sea_material,
            clay_material,
            hard_clay_material,
            bryce_pillars: StrictBool(bryce_pillars),
            has_forest: StrictBool(has_forest),
            extra: surface.extra,
        }
    }
}

impl NoiseGradientSurface {
    /// The noise gradient of `noise_descriptor` over `noise_block_specifiers`, replacing every
    /// block.
    #[must_use]
    pub fn new(
        noise_descriptor: NoiseDescriptor,
        noise_block_specifiers: Vec<NoiseBlockSpecifier>,
    ) -> Self {
        Self {
            noise_descriptor,
            non_replaceable_blocks: None,
            noise_block_specifiers,
            extra: Extra::default(),
        }
    }
}

impl NoiseDescriptor {
    /// The noise `name` from `first_octave` with one amplitude per octave.
    pub fn new(name: impl Into<String>, first_octave: i32, amplitudes: Vec<f32>) -> Self {
        Self {
            name: StrictString(name.into()),
            first_octave: StrictInt(first_octave),
            amplitudes: amplitudes.into_iter().map(StrictFloat).collect(),
            extra: Extra::default(),
        }
    }
}

impl NoiseBlockSpecifier {
    /// `block` with no noise, threshold or range written (they read as `""`, `0`, `{0, 0}`).
    #[must_use]
    pub fn new(block: BlockSpecifier) -> Self {
        Self {
            noise: None,
            threshold: None,
            range: None,
            block,
            extra: Extra::default(),
        }
    }
}

impl Adjustment {
    /// An adjustment setting `materials`, everywhere (no range, no scale).
    #[must_use]
    pub fn new(materials: AdjustmentMaterials) -> Self {
        Self {
            materials,
            noise_range: None,
            height_range: None,
            noise_frequency_scale: None,
            extra: Extra::default(),
        }
    }
}

impl BoolAttribute {
    /// `value`, combined by `operation`.
    #[must_use]
    pub fn new(value: bool, operation: BoolOperation) -> Self {
        Self {
            value: StrictBool(value),
            operation,
            extra: Extra::default(),
        }
    }
}

impl ColorAttribute {
    /// `value`, combined by `operation`.
    #[must_use]
    pub fn new(value: ColorRgba, operation: ColorOperation) -> Self {
        Self {
            value,
            operation,
            extra: Extra::default(),
        }
    }
}

impl FloatAttribute {
    /// `value`, combined by `operation`.
    #[must_use]
    pub fn new(value: f32, operation: FloatOperation) -> Self {
        Self {
            operation,
            value: StrictFloat(value),
            extra: Extra::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::component::{
        AdjustmentMaterials, BiomeComponent, Builder, CappedSurface, GrassTint, MesaSurface,
        NoiseBlockSpecifier, NoiseDescriptor, NoiseGradientSurface, OverworldSurface,
        ReplaceBiomes, SwampSurface, TheEndSurface,
    };
    use super::*;
    use bedrock_addon_core::document::Extra;
    use bedrock_addon_core::id::BlockId;
    use bedrock_addon_core::json::JsonSlice;
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::versioning::ConvertMode;

    /// Writes `component` and reads it back with its row parser: the same value, no finding.
    #[track_caller]
    fn reads_back(name: &str, component: BiomeComponent) {
        let json = super::super::component::write_component(&component).expect("writes");
        let text = bedrock_addon_core::json::to_facet_json(&json).expect("text");
        let row = BiomeComponent::TABLE.get(name).expect("a row");
        let mut found = bedrock_addon_core::diagnostic::Diagnostics::default();
        let read = (row.parse)(
            JsonSlice::new(&text),
            ConvertMode::Strict,
            &LoadContext::detached(),
            &mut found,
        );
        assert!(found.is_empty(), "{name}: {found:#?}");
        assert_eq!(read, Some(component), "{name}");
    }

    fn block(name: &'static str) -> BlockSpecifier {
        BlockSpecifier::new(BlockId::from_static(name))
    }

    fn overworld() -> OverworldSurface {
        OverworldSurface::new(
            7,
            block("minecraft:gravel"),
            block("minecraft:stone"),
            block("minecraft:dirt"),
            block("minecraft:grass_block"),
            block("minecraft:water"),
        )
    }

    #[test]
    fn every_constructor_builds_a_component_that_reads_back() {
        reads_back("minecraft:humidity", Humidity::new(true).into());
        reads_back("minecraft:tags", Tags::new(["animal", "plains"]).into());
        reads_back(
            "minecraft:mountain_parameters",
            super::super::component::MountainParameters {
                top_slide: Some(TopSlide::new(true)),
                ..Default::default()
            }
            .into(),
        );
        reads_back(
            "minecraft:replace_biomes",
            ReplaceBiomes::new(vec![BiomeReplacement::new(
                "minecraft:overworld",
                ["plains"],
                0.5,
                1.0,
            )])
            .into(),
        );
        reads_back(
            "minecraft:map_tints",
            MapTints::new(GrassTint::Noise {
                extra: Extra::default(),
            })
            .into(),
        );
        for builder in [
            Builder::Overworld(overworld()),
            Builder::Swamp(SwampSurface::new(overworld(), 3)),
            Builder::Capped(CappedSurface::new(
                vec![block("minecraft:netherrack")],
                vec![block("minecraft:netherrack")],
                block("minecraft:lava"),
                block("minecraft:netherrack"),
            )),
            Builder::TheEnd(TheEndSurface::default()),
            Builder::Mesa(MesaSurface::new(
                overworld(),
                block("minecraft:white_terracotta"),
                block("minecraft:hardened_clay"),
                false,
                true,
            )),
            Builder::NoiseGradient(NoiseGradientSurface::new(
                NoiseDescriptor::new("test:noise", -5, vec![1.0, 0.0]),
                vec![NoiseBlockSpecifier::new(block("minecraft:stone"))],
            )),
        ] {
            reads_back(
                "minecraft:surface_builder",
                SurfaceBuilder::new(builder.clone()).into(),
            );
            reads_back(
                "minecraft:subsurface_builder",
                SubsurfaceBuilder::new(builder).into(),
            );
        }
        reads_back(
            "minecraft:surface_material_adjustments",
            super::super::component::SurfaceMaterialAdjustments {
                adjustments: Some(vec![Adjustment::new(AdjustmentMaterials {
                    top_material: Some(block("minecraft:sand")),
                    ..Default::default()
                })]),
                ..Default::default()
            }
            .into(),
        );
    }

    #[test]
    fn the_attribute_constructors_read_back() {
        let attributes = super::super::EnvironmentAttributes {
            experimental_bool_attr: Some(BoolAttribute::new(true, BoolOperation::Xor)),
            experimental_color_attr: Some(ColorAttribute::new(
                ColorRgba::Rgba([1, 2, 3, 4]),
                ColorOperation::Multiply,
            )),
            experimental_float_attr_1: Some(FloatAttribute::new(0.5, FloatOperation::Maximum)),
            ..Default::default()
        };
        let text = bedrock_addon_core::json::to_json(&attributes).expect("writes");
        let read: super::super::EnvironmentAttributes =
            bedrock_addon_core::json::from_str(&text).expect("reads");
        assert_eq!(read, attributes);
    }
}
