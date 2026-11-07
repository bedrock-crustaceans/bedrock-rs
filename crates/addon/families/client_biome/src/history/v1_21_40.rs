//! Format 1.21.40: the first client-biome format, with seven components and identifiers that
//! may leave out the `minecraft:` namespace.

use bedrock_addon_derive::Components;
use facet::Facet;

use super::{ClientBiomePayload, has, insert, note_component, v1_21_70};
use crate::color::Color255Rgb;
use crate::component::{
    Checked, ClientBiomeComponent, DryFoliageColor, FoliageAppearance, FoliageColor,
    FoliageColorMap, GrassAppearance, GrassColor, GrassColorMap, child, parse_typed,
};
use crate::read::ComponentEra;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::{ComponentMap, ComponentSet, UnknownMembers};
use bedrock_addon_core::id::SoundEventName;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx, write_payload};

/// A client biome of format `[1.21.40, 1.21.70)`, read with its envelope: vanilla reads
/// `format_version` and `minecraft:client_biome` as members of one strict struct.
pub type ClientBiome = Legacy<ClientBiomePayload<Component>, 1, 21, 40>;

/// The key of `minecraft:ambient_sounds`.
const AMBIENT_SOUNDS: &str = "minecraft:ambient_sounds";

/// One component of a format below 1.21.120: `minecraft:ambient_sounds` in its first shape, or
/// a component of the newest set, whose shape the format already had (without the members and
/// the colour map later formats added).
#[derive(Clone, Debug, PartialEq, Components)]
#[component(from_only)]
#[non_exhaustive]
#[allow(
    clippy::large_enum_variant,
    reason = "a component map holds few, and boxing every variant would cost each access"
)]
pub enum Component {
    /// `minecraft:ambient_sounds` with a plain sound as `addition`.
    AmbientSounds(AmbientSounds),
    /// Any other component, in the newest shape.
    Newest(ClientBiomeComponent),
}

impl ComponentSet for Component {
    fn name(&self) -> Option<&'static str> {
        match self {
            Self::AmbientSounds(_) => Some(AMBIENT_SOUNDS),
            Self::Newest(component) => component.name(),
        }
    }
}

impl ComponentEra for Component {
    #[allow(
        clippy::option_option,
        reason = "three outcomes: not this era's, read, failed (reported)"
    )]
    fn read_own(
        key: &str,
        slice: JsonSlice<'_>,
        cx: &LoadContext,
        d: &mut Diagnostics,
    ) -> Option<Option<Self>> {
        (key == AMBIENT_SOUNDS).then(|| parse_typed::<AmbientSounds>(slice, cx, d).map(Self::from))
    }

    fn write(&self) -> Result<Value, EncodeError> {
        match self {
            Self::AmbientSounds(sounds) => write_payload(sounds),
            Self::Newest(component) => component.write_component(),
        }
    }
}

/// `minecraft:ambient_sounds` of formats `[1.21.40, 1.21.120)`: three sounds by name, every
/// member optional.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct AmbientSounds {
    /// JSON `addition`: a sound played now and then.
    pub addition: Option<SoundEventName>,
    /// JSON `loop`: the sound that plays all the time.
    #[facet(rename = "loop")]
    pub loop_sound: Option<SoundEventName>,
    /// JSON `mood`: the sound played in dark, enclosed places.
    pub mood: Option<SoundEventName>,
}

impl Checked for AmbientSounds {
    /// Each sound is a name: an `{asset, chance}` addition is the 1.21.120 struct's.
    fn check(json: &Value, _: Severity, d: &mut Diagnostics) {
        for member in ["addition", "loop", "mood"] {
            let sound = json.as_object().and_then(|object| object.get(member));
            if sound.is_some_and(|sound| !sound.is_string()) {
                d.push(
                    Diagnostic::of(&core_codes::INVALID_TYPE)
                        .at(child(&JsonPath::new(), &[member])),
                );
            }
        }
    }
}

/// The version below which a document gets the foliage and grass colours of the tables.
const COLOR_UPGRADE_BEFORE: FormatVersion = FormatVersion::new(1, 21, 60);
/// The version below which a document gets the dry foliage colour of the table.
const DRY_FOLIAGE_UPGRADE_BEFORE: FormatVersion = FormatVersion::new(1, 21, 70);

/// The five mesa biomes of the colour tables (`minecraft:mesa_plateau` is not one).
const MESAS: [&str; 5] = [
    "minecraft:mesa",
    "minecraft:mesa_bryce",
    "minecraft:mesa_plateau_mutated",
    "minecraft:mesa_plateau_stone",
    "minecraft:mesa_plateau_stone_mutated",
];

/// A fixed colour of the tables.
fn hex(text: &str) -> Color255Rgb {
    Color255Rgb::Hex(text.to_owned())
}

/// The foliage a vanilla biome's file left to code before 1.21.60.
fn foliage(identifier: &str) -> Option<FoliageColor> {
    let map = |color_map| FoliageColor::Map {
        color_map,
        extra: UnknownMembers::default(),
    };
    match identifier {
        id if MESAS.contains(&id) => Some(FoliageColor::Rgb(hex("#aea42a"))),
        "minecraft:cherry_grove" => Some(FoliageColor::Rgb(hex("#b6db61"))),
        "minecraft:pale_garden" => Some(FoliageColor::Rgb(hex("#878d76"))),
        "minecraft:swampland" | "minecraft:swampland_mutated" => {
            Some(map(FoliageColorMap::SwampFoliage))
        }
        "minecraft:mangrove_swamp" => Some(map(FoliageColorMap::MangroveSwampFoliage)),
        _ => None,
    }
}

/// The grass a vanilla biome's file left to code before 1.21.60.
fn grass(identifier: &str) -> Option<GrassColor> {
    match identifier {
        id if MESAS.contains(&id) => Some(GrassColor::Rgb(hex("#90814d"))),
        "minecraft:cherry_grove" => Some(GrassColor::Rgb(hex("#b6db61"))),
        "minecraft:pale_garden" => Some(GrassColor::Rgb(hex("#778272"))),
        "minecraft:swampland" | "minecraft:swampland_mutated" | "minecraft:mangrove_swamp" => {
            Some(GrassColor::Map {
                color_map: GrassColorMap::SwampGrass,
                extra: UnknownMembers::default(),
            })
        }
        _ => None,
    }
}

/// The dry foliage a vanilla biome's file left to code before 1.21.70.
fn dry_foliage(identifier: &str) -> Option<Color255Rgb> {
    match identifier {
        "minecraft:swampland"
        | "minecraft:swampland_mutated"
        | "minecraft:mangrove_swamp"
        | "minecraft:roofed_forest"
        | "minecraft:roofed_forest_mutated" => Some(hex("#7b5334")),
        "minecraft:pale_garden" => Some(hex("#a0a69c")),
        _ => None,
    }
}

/// Inserts the component `name` a table gives the identifier, when the document has none.
/// The tables are keyed `minecraft:…` and vanilla compares the identifier as written, so a
/// vanilla biome named without its namespace gets nothing; the crate says which table it
/// missed.
fn from_table<T>(
    components: &mut ComponentMap<Component>,
    identifier: &str,
    table: impl Fn(&str) -> Option<T>,
    cx: &mut UpgradeCx<'_>,
) where
    T: bedrock_addon_core::document::Component<Set = ClientBiomeComponent>,
{
    if has(components, T::NAME) {
        return;
    }
    if let Some(component) = table(identifier) {
        insert(components, component, cx);
    } else if table(&format!("minecraft:{identifier}")).is_some() {
        note_component(
            cx,
            DiagnosticCode::from(&core_codes::UPGRADE_TABLE_MISSED_BARE_IDENTIFIER),
            T::NAME,
        );
    }
}

/// The step to 1.21.70: everything moves
/// (`foliage_appearance` and `biome_music` into types that only gain members), then an
/// override of a vanilla biome gets the foliage, grass and dry-foliage colours 1.21.60 and
/// 1.21.70 moved from code into data.
impl LegacyStep<1, 21, 40> for ClientBiomePayload<Component> {
    type Next = v1_21_70::ClientBiome;

    fn step(this: ClientBiome, cx: &mut UpgradeCx<'_>) -> v1_21_70::ClientBiome {
        let biome = this.0;
        let mut components = biome.components;
        let identifier = biome.description.identifier.as_str().to_owned();
        if cx.original_version < COLOR_UPGRADE_BEFORE {
            from_table(
                &mut components,
                &identifier,
                |id| foliage(id).map(|color| FoliageAppearance { color: Some(color) }),
                cx,
            );
            from_table(
                &mut components,
                &identifier,
                |id| {
                    // Vanilla allocates the 1.21.100 type with `grass_is_shaded = false`; the
                    // member stays absent here and reads `false`.
                    grass(id).map(|color| GrassAppearance {
                        color: Some(color),
                        grass_is_shaded: None,
                    })
                },
                cx,
            );
        }
        // Every document this struct reads is below 1.21.70; the gate is vanilla's.
        if cx.original_version < DRY_FOLIAGE_UPGRADE_BEFORE {
            from_table(
                &mut components,
                &identifier,
                |id| dry_foliage(id).map(|color| DryFoliageColor { color }),
                cx,
            );
        }
        Legacy(ClientBiomePayload {
            format_version: biome.format_version,
            description: biome.description,
            components,
        })
    }
}
