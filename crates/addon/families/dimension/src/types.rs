//! The dimension document's types and their inherent items.

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, JsonPath, Phase};
use bedrock_addon_core::document::Document;
use bedrock_addon_core::id::{BiomeId, DimensionId};
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::version::FormatVersion;

/// A dimension document (`minecraft:dimension`), at the newest format version.
///
/// JSON: `{"format_version": "1.26.50", "minecraft:dimension": {"description", "components"}}`.
/// A vanilla dimension (`minecraft:overworld`, `minecraft:nether`, `minecraft:the_end`) takes
/// no default biome and no clouds, and the nether and the end no height; any other identifier
/// is a custom dimension in a namespace other than `minecraft`.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:dimension")]
#[doc(alias = "dimensions")]
pub struct Dimension {
    /// Which dimension the document configures (JSON `description`).
    pub description: DimensionDescription,
    /// Its height, default biome and generator (JSON `components`).
    pub components: DimensionComponents,
    /// The document's `format_version` (the floor when absent). Not a JSON member; the writer
    /// writes it as `format_version` when it is 1.26.50 or newer.
    pub source_format_version: FormatVersion,
}

/// `minecraft:dimension/description`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DimensionDescription {
    /// The dimension (JSON `identifier`): one of the three vanilla dimensions, exactly, or a
    /// custom dimension `namespace:name`.
    pub identifier: DimensionId,
}

/// `minecraft:dimension/components` from format 1.26.50: a fixed set of members, not a
/// component registry.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DimensionComponents {
    /// The height (JSON `minecraft:dimension_height`).
    #[facet(rename = "minecraft:dimension_height")]
    pub dimension_height: Option<DimensionHeight>,
    /// The biome generated where no other is (JSON `minecraft:default_biome`).
    #[facet(rename = "minecraft:default_biome")]
    pub default_biome: Option<DefaultBiome>,
    /// The cloud height (JSON `minecraft:clouds`; a custom dimension's only).
    #[facet(rename = "minecraft:clouds")]
    pub clouds: Option<Clouds>,
    /// The generator (JSON `minecraft:generation`).
    #[facet(rename = "minecraft:generation")]
    pub generation: Generation,
}

/// `minecraft:dimension_height`: where the dimension starts and how tall it is.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DimensionHeight {
    /// The lowest y (JSON `min_y`), in `[-512, 512]`.
    #[facet(schema::min = -512, schema::max = 512)]
    pub min_y: i16,
    /// The height (JSON `height_range`), in `[16, 1024]` and a multiple of 16.
    #[facet(schema::min = 16, schema::max = 1024, schema::multiple_of = 16)]
    pub height_range: i16,
}

impl DimensionHeight {
    /// `(min_y, height_range)`.
    pub(crate) fn pair(&self) -> (i16, i16) {
        (self.min_y, self.height_range)
    }

    /// Whether the newest format can write this height: not `{0, 0}`, the height the upgrade
    /// makes of absent or zero bounds, which the bind constraint of `height_range` (`[16,
    /// 1024]`) would refuse when read back; the writer leaves such a height out.
    pub(crate) fn is_writable(&self) -> bool {
        self.pair() != (0, 0)
    }
}

/// `minecraft:default_biome`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DefaultBiome {
    /// JSON `biome`.
    pub biome: BiomeId,
}

/// `minecraft:clouds`: "Defines the cloud height in the custom dimension."
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Clouds {
    /// "The Y coordinate at which clouds are rendered in the custom dimension. Defaults to 128
    /// if this component is omitted." (JSON `height`, required), in `[-512, 512]`.
    #[facet(schema::min = -512, schema::max = 512)]
    pub height: i16,
}

/// `minecraft:generation`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Generation {
    /// The generator as written (JSON `generator_type`); only `void` names one (see
    /// [`Generation::generator`]), any other text is a warning and the document loads.
    pub generator_type: String,
}

impl Dimension {
    /// The height of a dimension without `minecraft:dimension_height`: `{min_y −64, height_range
    /// 384}`, which both the validator and the runtime definition use.
    pub const DEFAULT_HEIGHT: (i16, i16) = (-64, 384);

    /// The cloud height of a dimension without `minecraft:clouds`.
    pub const DEFAULT_CLOUD_HEIGHT: i16 = 128;

    /// The default biome of a dimension without `minecraft:default_biome`.
    pub const DEFAULT_BIOME: BiomeId = BiomeId::from_static("minecraft:ocean");

    /// `(min_y, height_range)` of the runtime definition the game builds of this document: the
    /// height, or [`Self::DEFAULT_HEIGHT`] when there is none.
    pub fn runtime_height(&self) -> (i16, i16) {
        self.components
            .dimension_height
            .as_ref()
            .map_or(Self::DEFAULT_HEIGHT, DimensionHeight::pair)
    }

    /// The default biome of the runtime definition: `minecraft:default_biome.biome`, or
    /// [`Self::DEFAULT_BIOME`] (`minecraft:ocean`).
    pub fn runtime_default_biome(&self) -> &BiomeId {
        static OCEAN: BiomeId = Dimension::DEFAULT_BIOME;
        self.components
            .default_biome
            .as_ref()
            .map_or(&OCEAN, |default| &default.biome)
    }

    /// The cloud height of the runtime definition: `minecraft:clouds.height`, or
    /// [`Self::DEFAULT_CLOUD_HEIGHT`] (128).
    pub fn runtime_cloud_height(&self) -> i16 {
        self.components
            .clouds
            .as_ref()
            .map_or(Self::DEFAULT_CLOUD_HEIGHT, |clouds| clouds.height)
    }
}

impl Generation {
    /// The generator `generator_type` names: `void` is [`GeneratorType::Void`]; any other text
    /// (case-sensitive) names none.
    // The game's table of generator names holds the single entry `void`.
    pub fn generator(&self) -> Option<GeneratorType> {
        (self.generator_type == "void").then_some(GeneratorType::Void)
    }
}

/// `minecraft:dimension_bounds` of a document below 1.26.50: the lowest and highest y.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DimensionBounds {
    /// JSON `min`, in `[-512, 512]` and a multiple of 16.
    #[facet(schema::min = -512, schema::max = 512, schema::multiple_of = 16)]
    pub min: i32,
    /// JSON `max`, in `[-512, 512]` and a multiple of 16.
    #[facet(schema::min = -512, schema::max = 512, schema::multiple_of = 16)]
    pub max: i32,
}

impl DimensionBounds {
    /// Bounds `[min, max]`.
    pub fn new(min: i32, max: i32) -> Self {
        Self { min, max }
    }
}

/// A world generator (vanilla `GeneratorType`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum GeneratorType {
    /// `Legacy` (0).
    Legacy,
    /// `Overworld` (1).
    Overworld,
    /// `Flat` (2).
    Flat,
    /// `Nether` (3).
    Nether,
    /// `TheEnd` (4).
    TheEnd,
    /// `Void` (5): the only one a document can name.
    Void,
    /// `Undefined` (6).
    Undefined,
}

impl From<GeneratorType> for u8 {
    fn from(generator: GeneratorType) -> Self {
        generator.id()
    }
}

impl GeneratorType {
    /// Vanilla's numeric value.
    pub fn id(self) -> u8 {
        match self {
            Self::Legacy => 0,
            Self::Overworld => 1,
            Self::Flat => 2,
            Self::Nether => 3,
            Self::TheEnd => 4,
            Self::Void => 5,
            Self::Undefined => 6,
        }
    }
}

impl Dimension {
    /// The warning the game gives when it cannot create the custom dimension `document`
    /// defines at world start, when the runtime fails to register it: `InvalidValue` (warning)
    /// "Dimension '{}' from pack id '{}' was invalid and could not be created!" with the
    /// identifier and the pack's UUID, at the document's identifier. The game writes the line
    /// at the content log's warning level. The failure is the runtime's, which no document
    /// rule predicts; a host that creates dimensions reports it with this.
    pub fn creation_failed(document: &Document<Dimension>) -> Diagnostic {
        let origin = document.origin();
        let at = JsonPath::new().with(&["minecraft:dimension", "description", "identifier"]);
        // The game logs this at the content log's warning level.
        Diagnostic::warn(DiagnosticCode::InvalidValue {
            text: Some(bedrock_addon_core::diagnostic::VanillaText::Formatted {
                template: "Dimension '{}' from pack id '{}' was invalid and could not be created!",
                args: vec![
                    document.description.identifier.as_str().to_owned(),
                    origin.pack.id().uuid.to_string(),
                ],
            }),
        })
        .at(at)
        .phase(Phase::Validate)
        .in_file(origin)
    }
}
