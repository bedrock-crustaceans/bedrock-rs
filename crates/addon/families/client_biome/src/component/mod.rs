//! The components of a client biome (`minecraft:client_biome/components`): one type per
//! component, named after the JSON key without its namespace, in `PascalCase`.
//!
//! [`ClientBiomeComponent`] is the closed set a [`bedrock_addon_core::document::ComponentMap`] of a client
//! biome holds; `biome.components.get::<SkyColor>()` reads one. A key the set does not know is
//! reported and kept as written; vanilla rejects such a document.

mod appearance;
mod identifier;
mod sound;

pub use appearance::{
    DryFoliageColor, FoliageAppearance, FoliageColor, FoliageColorMap, GrassAppearance, GrassColor,
    GrassColorMap, Precipitation, SkyColor, WaterAppearance,
};
pub use identifier::{
    AtmosphereIdentifier, ColorGradingIdentifier, CubemapIdentifier, FogAppearance,
    LightingIdentifier, WaterIdentifier,
};
pub use sound::{AmbientSounds, BiomeMusic, SoundAddition};

use bedrock_addon_derive::Components;
use facet::Facet;

use super::history::ClientBiomeFamily;
use bedrock_addon_core::diagnostic::{
    Class, Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity,
};
use bedrock_addon_core::json::{
    JsonSlice, ScalarFailure, ScalarRead, Value, keep_unknown_members, parse_with_rules,
    unknown_members,
};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::versioning::ConvertMode;

// Every component is released with the first format version whose document declares it:
// 1.21.40 declares seven, 1.21.70 adds five (the atmosphere, colour-grading, lighting and
// water identifiers and `minecraft:dry_foliage_color`), 1.21.110 adds `minecraft:precipitation`
// and 1.21.130 adds `minecraft:cubemap_identifier`. The 1.21.130 document holds all fourteen,
// each with the type of the version that last changed it. No component has a requirement, a
// deprecation or a non-public area; `released` is the first version that binds it.

/// One component of a client biome: the closed set of `minecraft:` components the 1.21.130
/// document declares.
#[derive(Clone, Debug, PartialEq, Components)]
#[component(family = ClientBiomeFamily, write)]
#[non_exhaustive]
#[allow(
    clippy::large_enum_variant,
    reason = "a component map holds few, and boxing every variant would cost each access"
)]
pub enum ClientBiomeComponent {
    /// `minecraft:ambient_sounds`.
    #[component(name = "minecraft:ambient_sounds", released = "1.21.40", parse = parse_row::<AmbientSounds>)]
    AmbientSounds(AmbientSounds),
    /// `minecraft:atmosphere_identifier` (format 1.21.70).
    #[component(name = "minecraft:atmosphere_identifier", released = "1.21.70", parse = parse_row::<AtmosphereIdentifier>)]
    AtmosphereIdentifier(AtmosphereIdentifier),
    /// `minecraft:biome_music`.
    #[component(name = "minecraft:biome_music", released = "1.21.40", parse = parse_row::<BiomeMusic>)]
    BiomeMusic(BiomeMusic),
    /// `minecraft:color_grading_identifier` (format 1.21.70).
    #[component(name = "minecraft:color_grading_identifier", released = "1.21.70", parse = parse_row::<ColorGradingIdentifier>)]
    ColorGradingIdentifier(ColorGradingIdentifier),
    /// `minecraft:cubemap_identifier` (format 1.21.130).
    #[component(name = "minecraft:cubemap_identifier", released = "1.21.130", parse = parse_row::<CubemapIdentifier>)]
    CubemapIdentifier(CubemapIdentifier),
    /// `minecraft:dry_foliage_color` (format 1.21.70).
    #[component(name = "minecraft:dry_foliage_color", released = "1.21.70", parse = parse_row::<DryFoliageColor>)]
    DryFoliageColor(DryFoliageColor),
    /// `minecraft:fog_appearance`.
    #[component(name = "minecraft:fog_appearance", released = "1.21.40", parse = parse_row::<FogAppearance>)]
    FogAppearance(FogAppearance),
    /// `minecraft:foliage_appearance`.
    #[component(name = "minecraft:foliage_appearance", released = "1.21.40", parse = parse_row::<FoliageAppearance>)]
    FoliageAppearance(FoliageAppearance),
    /// `minecraft:grass_appearance`.
    #[component(name = "minecraft:grass_appearance", released = "1.21.40", parse = parse_row::<GrassAppearance>)]
    GrassAppearance(GrassAppearance),
    /// `minecraft:lighting_identifier` (format 1.21.70).
    #[component(name = "minecraft:lighting_identifier", released = "1.21.70", parse = parse_row::<LightingIdentifier>)]
    LightingIdentifier(LightingIdentifier),
    /// `minecraft:precipitation` (format 1.21.110).
    #[component(name = "minecraft:precipitation", released = "1.21.110", parse = parse_row::<Precipitation>)]
    Precipitation(Precipitation),
    /// `minecraft:sky_color`.
    #[component(name = "minecraft:sky_color", released = "1.21.40", parse = parse_row::<SkyColor>)]
    SkyColor(SkyColor),
    /// `minecraft:water_appearance`.
    #[component(name = "minecraft:water_appearance", released = "1.21.40", parse = parse_row::<WaterAppearance>)]
    WaterAppearance(WaterAppearance),
    /// `minecraft:water_identifier` (format 1.21.70).
    #[component(name = "minecraft:water_identifier", released = "1.21.70", parse = parse_row::<WaterIdentifier>)]
    WaterIdentifier(WaterIdentifier),
}

/// The constraints vanilla's binding attaches to a component, checked on its JSON before the
/// typed parse so that each failure carries its own code and vanilla's text.
pub(crate) trait Checked {
    /// Reports the component's constraint failures (paths below the component); an error
    /// among them rejects the component. An unknown member it finds is told at `unknown`, the
    /// slice's grade.
    #[allow(unused_variables)]
    fn check(json: &Value, unknown: Severity, d: &mut Diagnostics) {}
}

/// One component read strictly from its slice: every slice of the client-biome loader parses
/// with the strict mask, so a component that does not deserialise, or breaks a constraint of
/// its binding, rejects the document. Unknown members are reported as `UnrecognizedField`,
/// graded by the slice's reading: every client-biome format version reads with the strict mask
/// (`Family::reading`'s default, the profile of the `InputKind::Document` envelope in `lib.rs`),
/// so in a load they are the errors that refuse the document, as vanilla's are. Findings are
/// relative to the component.
pub(crate) fn parse_typed<T>(
    slice: JsonSlice<'_>,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<T>
where
    T: for<'a> Facet<'a> + Checked,
{
    let json = bedrock_addon_core::json::from_str_jsonc::<Value>(slice.text).ok();
    let unknown = cx.grade(Class::Unknown);
    let unrecognized = |path: JsonPath| {
        Diagnostic::warn(DiagnosticCode::UnrecognizedField { text: None })
            .at(path)
            .severity(unknown)
    };
    if let Some(json) = &json {
        // A component is an object; facet's own finding for another value has no path.
        if !json.is_object() {
            d.push(Diagnostic::error(DiagnosticCode::InvalidType {
                text: None,
            }));
            return None;
        }
        let start = d.len();
        T::check(json, unknown, d);
        // An unknown member does not stop the read here, the checks' other errors alone do: it
        // refuses the document by its grade.
        if d.iter()
            .skip(start)
            .any(|f| f.severity == Severity::Error && !crate::read::is_unknown_member(f))
        {
            return None;
        }
    }
    // The scalars are read by the family's rules (`crate::reading`: strict, a number beyond its
    // float a type error), in a versioned load and outside one alike.
    // A `null` read as a struct is the null rule's: the row parser's `reject_nulls` tells it
    // where the input rejects nulls, and elsewhere it reads as its defaults untold.
    let rules = crate::reading()
        .scalar_rules()
        .with_tells_null_struct(false);
    let value = match parse_with_rules::<T>(slice, None, rules) {
        Ok(ScalarRead { value, told, .. }) => {
            // Under the strict rules the pre-pass refuses every conversion, so it tells nothing.
            for misread in told {
                d.push(misread.finding(&JsonPath::new()));
            }
            value
        }
        Err(ScalarFailure::Refused(misread)) => {
            d.push(misread.finding(&JsonPath::new()));
            for (path, _) in json.iter().flat_map(|json| unknown_members(T::SHAPE, json)) {
                d.push(unrecognized(path));
            }
            return None;
        }
        Err(ScalarFailure::Read(error)) => {
            d.push(error.into_diagnostic(&JsonPath::new(), |code, _| code));
            // The members no field takes are reported also when the component fails, so a
            // document of another era names what it has that the struct does not.
            for (path, _) in json.iter().flat_map(|json| unknown_members(T::SHAPE, json)) {
                d.push(unrecognized(path));
            }
            return None;
        }
    };
    if let Some(json) = &json {
        let mut kept = bedrock_addon_core::document::Unknown::new();
        for (path, _) in keep_unknown_members(&value, json, &mut kept) {
            d.push(unrecognized(path));
        }
        // Below the component; the component map puts them at its key.
        d.unknown_mut().absorb(&JsonPath::new(), kept);
    }
    Some(value)
}

/// The table's row parser: [`parse_typed`] into the set.
fn parse_row<T>(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<ClientBiomeComponent>
where
    T: for<'a> Facet<'a> + Checked + Into<ClientBiomeComponent>,
{
    let _ = mode;
    let value = parse_typed::<T>(slice, cx, d)?;
    // A `null` where the component reads a value is a type error, which refuses the client
    // biome.
    if cx.null_rule == Some(bedrock_addon_core::json::NullRule::Rejected)
        && bedrock_addon_core::json::reject_nulls(&value, slice, &JsonPath::new(), d)
    {
        return None;
    }
    Some(value.into())
}

pub(crate) use bedrock_addon_support::biome_json::child;
