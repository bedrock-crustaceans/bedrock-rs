//! Reading the payload object of every biome struct: `description`, `components` (one at a
//! time, by the components the struct's version binds) and, in the beta document,
//! `environment_attributes`.

use facet::Facet;
use facet_json::RawJson;
use indexmap::IndexMap;

use super::component::{BiomeComponent, Builder, SubsurfaceBuilder, SurfaceBuilder, child};
use super::{Description, EnvironmentAttributes};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath};
use bedrock_addon_core::document::{ComponentMap, Extra};
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{ConvertMode, ParseCx, parse_payload};

/// The first version read from the `minecraft:biome` payload alone (below it the whole
/// document, envelope included, is read as one struct).
pub(crate) const CEREAL: FormatVersion = known::V1_21_120;
/// The first struct with `minecraft:village_type` and the nether replacement.
pub(crate) const V1_26_0: FormatVersion = known::V1_26_0;
/// The first struct with `minecraft:subsurface_builder` and the noise-gradient builder.
pub(crate) const V1_26_30: FormatVersion = known::V1_26_30;

/// A stable payload: components as raw slices, parsed one at a time.
#[derive(Facet)]
struct Shell<'a> {
    description: Description,
    components: IndexMap<String, RawJson<'a>>,
    #[facet(skip, rename = "\u{0}extra")]
    extra: Extra,
}

/// The beta payload.
#[derive(Facet)]
struct BetaShell<'a> {
    description: Description,
    components: IndexMap<String, RawJson<'a>>,
    #[facet(default)]
    environment_attributes: Option<EnvironmentAttributes>,
    #[facet(skip, rename = "\u{0}extra")]
    extra: Extra,
}

/// What a payload object holds.
pub(crate) struct Payload {
    pub(crate) description: Description,
    pub(crate) components: ComponentMap<BiomeComponent>,
}

/// Reads a stable payload object as the struct of `version` binds it.
pub(crate) fn payload(
    slice: JsonSlice<'_>,
    version: FormatVersion,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<Payload> {
    let shell: Shell<'_> = parse_payload(slice, cx, d)?;
    let components = components(
        &shell.components,
        slice,
        &child(&cx.at, &["components"]),
        version,
        cx.load,
        d,
    );
    Some(Payload {
        description: shell.description,
        components,
    })
}

/// Reads the beta payload object: the 1.26.30 components and `environment_attributes`.
pub(crate) fn beta_payload(
    slice: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<(Payload, Option<EnvironmentAttributes>)> {
    let shell: BetaShell<'_> = parse_payload(slice, cx, d)?;
    let components = components(
        &shell.components,
        slice,
        &child(&cx.at, &["components"]),
        bedrock_addon_core::versioning::BETA_VERSION,
        cx.load,
        d,
    );
    let payload = Payload {
        description: shell.description,
        components,
    };
    Some((payload, shell.environment_attributes))
}

/// The `components` object of a struct of `version`, in file order. A component the struct
/// binds is read strictly (a failure is an error). Any other key, a component of a later
/// struct (`minecraft:village_type` before 1.26.0, `minecraft:subsurface_builder` before
/// 1.26.30) included, is an `UnrecognizedComponent` and dropped; the family's strict mask
/// makes it an error that refuses the document, as vanilla does for an unknown member in
/// every format version.
pub(crate) fn components(
    entries: &IndexMap<String, RawJson<'_>>,
    within: JsonSlice<'_>,
    at: &JsonPath,
    version: FormatVersion,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> ComponentMap<BiomeComponent> {
    let table = BiomeComponent::TABLE;
    let mut map = ComponentMap::new();
    for (key, raw) in entries {
        let slice = within.nested(raw).unwrap_or(JsonSlice {
            text: raw.as_str(),
            offset: within.offset,
        });
        let path = at.child(key);
        let meta = table.get(key);
        let bound = meta.is_some_and(|m| m.released.is_some_and(|r| version >= r));
        match meta {
            Some(meta) if bound => {
                let mut found = Diagnostics::default();
                let value = (meta.parse)(slice, ConvertMode::Strict, cx, &mut found);
                let kept = found.take_unknown();
                for mut diagnostic in found {
                    let mut full = path.clone();
                    full.append(&diagnostic.location.json_path);
                    diagnostic.location.json_path = full;
                    d.push(diagnostic);
                }
                if let Some(value) = value {
                    // The component's unknown members, below its key.
                    d.unknown_mut().absorb(&path, kept);
                    builder_bound(&value, version, &path, d);
                    map.insert_typed(value, false);
                }
            }
            _ => d.push(unrecognized(path)),
        }
    }
    map
}

/// Below 1.26.30 the builder variant has no `minecraft:noise_gradient` alternative: such a
/// builder is an unknown `type`, which the strict slice rejects.
fn builder_bound(
    value: &BiomeComponent,
    version: FormatVersion,
    at: &JsonPath,
    d: &mut Diagnostics,
) {
    let (BiomeComponent::SurfaceBuilder(SurfaceBuilder { builder, .. })
    | BiomeComponent::SubsurfaceBuilder(SubsurfaceBuilder { builder, .. })) = value
    else {
        return;
    };
    if version < V1_26_30 && matches!(builder, Builder::NoiseGradient(_)) {
        d.push(
            Diagnostic::error(DiagnosticCode::InvalidEnumValue { text: None })
                .at(child(at, &["builder", "type"])),
        );
    }
}

/// An `UnrecognizedComponent` at `path`: the strict mask's input, not a grade. Core's
/// `unknown_members_are_errors` (`versioning/pipeline.rs`) turns this Parse-phase warning into the
/// refusing error the game gives for an unknown member.
fn unrecognized(path: JsonPath) -> Diagnostic {
    Diagnostic::warn(DiagnosticCode::UnrecognizedComponent { text: None }).at(path)
}
