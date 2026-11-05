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
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
    code_for_deserialize_error_kind,
};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, Value, keep_unknown_members, parse_at, unknown_members,
};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::versioning::{ConvertMode, write_payload};

// Every component is released with the first format version whose document declares it:
// 1.21.40 declares seven, 1.21.70 adds five (the atmosphere, colour-grading, lighting and
// water identifiers and `minecraft:dry_foliage_color`), 1.21.110 adds `minecraft:precipitation`
// and 1.21.130 adds `minecraft:cubemap_identifier`. The 1.21.130 document holds all fourteen,
// each with the type of the version that last changed it. No component has a requirement, a
// deprecation or a non-public area; `released` is the first version that binds it.

/// One component of a client biome: the closed set of `minecraft:` components the 1.21.130
/// document declares.
#[derive(Clone, Debug, PartialEq, Components)]
#[component(family = ClientBiomeFamily)]
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
    /// Reports the component's constraint failures (paths below the component).
    #[allow(unused_variables)]
    fn check(json: &Value, report: &mut Report<'_>) {}
}

/// Where a component's checks report, at paths below the component.
pub(crate) struct Report<'a> {
    diagnostics: &'a mut Diagnostics,
    errors: usize,
}

impl<'a> Report<'a> {
    /// A report into `diagnostics`.
    pub(crate) fn new(diagnostics: &'a mut Diagnostics) -> Self {
        Self {
            diagnostics,
            errors: 0,
        }
    }

    /// An error at `steps` below the component: the document is not loaded.
    pub(crate) fn error(&mut self, code: DiagnosticCode, steps: &[&str]) {
        self.errors += 1;
        self.push(code, Severity::Error, steps);
    }

    /// An information at `steps` below the component: a finding of the crate's own that the
    /// game has no line for.
    pub(crate) fn info(&mut self, code: DiagnosticCode, steps: &[&str]) {
        self.push(code, Severity::Info, steps);
    }

    /// An unknown member at `steps` below the component. Not a grade the user sees: every
    /// client-biome format version reads with the strict mask (`error_mask` in `load.rs`), and
    /// core's `unknown_members_are_errors` (`versioning/pipeline.rs`) turns this Parse-phase
    /// `UnrecognizedField` warning into the refusing error the game gives for an unknown
    /// member; the `Warning` is that step's input.
    pub(crate) fn unknown_member(&mut self, code: DiagnosticCode, steps: &[&str]) {
        self.push(code, Severity::Warning, steps);
    }

    /// Whether an error was reported.
    pub(crate) fn failed(&self) -> bool {
        self.errors > 0
    }

    fn push(&mut self, code: DiagnosticCode, severity: Severity, steps: &[&str]) {
        self.diagnostics.push(Diagnostic::new(
            code,
            severity,
            Phase::Parse,
            Location::new(None, None, child(&JsonPath::new(), steps), None),
        ));
    }
}

/// One component read strictly from its slice: every slice of the client-biome loader parses
/// with the strict mask, so a component that does not deserialise, or breaks a constraint of
/// its binding, rejects the document. Unknown members are reported as `UnrecognizedField`,
/// which the family's strict mask makes errors that refuse the document, as vanilla does.
/// Findings are relative to the component.
pub(crate) fn parse_typed<T>(slice: JsonSlice<'_>, d: &mut Diagnostics) -> Option<T>
where
    T: for<'a> Facet<'a> + Checked,
{
    let json = bedrock_addon_core::json::from_str_jsonc::<Value>(slice.text).ok();
    let unrecognized = |path: JsonPath| {
        // Not a finding the user sees as a warning: every client-biome format version reads
        // with the strict mask (`error_mask` in `load.rs`), and core's
        // `unknown_members_are_errors` (`versioning/pipeline.rs`) turns a Parse-phase
        // `UnrecognizedField` / `UnrecognizedComponent` warning into the refusing error the game
        // gives for an unknown member. The `Warning` here is that step's input, not a grade.
        Diagnostic::new(
            DiagnosticCode::UnrecognizedField { text: None },
            Severity::Warning,
            Phase::Parse,
            Location::new(None, None, path, None),
        )
    };
    if let Some(json) = &json {
        let mut report = Report::new(d);
        // A component is an object; facet's own finding for another value has no path.
        if !json.is_object() {
            report.error(DiagnosticCode::InvalidType { text: None }, &[]);
            return None;
        }
        T::check(json, &mut report);
        if report.failed() {
            return None;
        }
    }
    let mut value = match parse_at::<T>(slice) {
        Ok(value) => value,
        Err(error) => {
            let blank = slice.text.trim().is_empty();
            let code =
                code_for_deserialize_error_kind(&error.error.kind, !error.path.is_empty(), blank);
            d.push(Diagnostic::new(
                code,
                Severity::Error,
                Phase::Parse,
                Location::new(None, None, error.path.clone(), Some(error.span)),
            ));
            // The members no field takes are reported also when the component fails, so a
            // document of another era names what it has that the struct does not.
            for (path, _) in json.iter().flat_map(|json| unknown_members(T::SHAPE, json)) {
                d.push(unrecognized(path));
            }
            return None;
        }
    };
    if let Some(json) = &json {
        for (path, _) in keep_unknown_members(&mut value, json) {
            d.push(unrecognized(path));
        }
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
    let value = parse_typed::<T>(slice, d)?;
    // A `null` where the component reads a value is a type error, which refuses the client
    // biome.
    if cx.null_rule == Some(bedrock_addon_core::json::NullRule::Rejected)
        && bedrock_addon_core::json::reject_nulls(&value, slice, &JsonPath::new(), d)
    {
        return None;
    }
    Some(value.into())
}

/// One typed component as JSON.
pub(crate) fn write_component(component: &ClientBiomeComponent) -> Result<Value, EncodeError> {
    use ClientBiomeComponent as C;
    match component {
        C::AmbientSounds(c) => write_payload(c),
        C::AtmosphereIdentifier(c) => write_payload(c),
        C::BiomeMusic(c) => write_payload(c),
        C::ColorGradingIdentifier(c) => write_payload(c),
        C::CubemapIdentifier(c) => write_payload(c),
        C::DryFoliageColor(c) => write_payload(c),
        C::FogAppearance(c) => write_payload(c),
        C::FoliageAppearance(c) => write_payload(c),
        C::GrassAppearance(c) => write_payload(c),
        C::LightingIdentifier(c) => write_payload(c),
        C::Precipitation(c) => write_payload(c),
        C::SkyColor(c) => write_payload(c),
        C::WaterAppearance(c) => write_payload(c),
        C::WaterIdentifier(c) => write_payload(c),
    }
}

pub(crate) use bedrock_addon_support::biome_json::child;
