//! Reading the payload object of every biome struct: `description`, `components` (one at a
//! time, by the components the struct's version binds) and, in the beta document,
//! `environment_attributes`.

use facet::Facet;
use facet_json::RawJson;
use indexmap::IndexMap;

use super::component::{BiomeComponent, Builder, SubsurfaceBuilder, SurfaceBuilder, child};
use super::{Description, EnvironmentAttributes};
use bedrock_addon_core::diagnostic::{Class, Diagnostic, DiagnosticCode, Diagnostics, JsonPath};
use bedrock_addon_core::document::ComponentMap;
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::schema;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{ConvertMode, ParseCx, parse_payload};
use bedrock_addon_shared_biome_env::read_component;

/// The first version read from the `minecraft:biome` payload alone (below it the whole
/// document, envelope included, is read as one struct).
pub(crate) const CEREAL: FormatVersion = known::V1_21_120;
/// The first struct with `minecraft:village_type` and the nether replacement.
pub(crate) const V1_26_0: FormatVersion = known::V1_26_0;
/// The first struct with `minecraft:subsurface_builder` and the noise-gradient builder.
pub(crate) const V1_26_30: FormatVersion = known::V1_26_30;

/// A stable payload: components as raw slices, parsed one at a time.
#[derive(Facet)]
#[facet(schema::unknown = "keep")]
struct Shell<'a> {
    description: Description,
    components: IndexMap<String, RawJson<'a>>,
}

/// The beta payload.
#[derive(Facet)]
#[facet(schema::unknown = "keep")]
struct BetaShell<'a> {
    description: Description,
    components: IndexMap<String, RawJson<'a>>,
    environment_attributes: Option<EnvironmentAttributes>,
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
/// 1.26.30) included, is an `UnrecognizedComponent` and dropped, graded by the slice's reading:
/// the family's strict mask makes it an error that refuses the document, as vanilla does for an
/// unknown member in every format version.
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
                let value = read_component(&path, d, |found| {
                    (meta.parse)(slice, ConvertMode::Strict, cx, found)
                });
                if let Some(value) = value {
                    builder_bound(&value, version, &path, d);
                    map.insert_typed(value, false);
                }
            }
            _ => d.push(unrecognized(path, cx)),
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

/// An `UnrecognizedComponent` at `path`, graded by the slice's reading (the refusing error the
/// game gives for an unknown member under the strict mask).
fn unrecognized(path: JsonPath, cx: &LoadContext) -> Diagnostic {
    Diagnostic::warn(DiagnosticCode::UnrecognizedComponent { text: None })
        .at(path)
        .severity(cx.grade(Class::Unknown))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::document::ComponentRef;

    /// An unknown component key is told and left out of the map (`client_biome` keeps it as an
    /// unknown entry); the strict mask refuses the document either way, so only this read
    /// shows the difference.
    #[test]
    fn an_unknown_component_key_is_dropped_from_the_map() {
        let text = r#"{"demo:nothing": {"a": 1}, "minecraft:tags": {"tags": ["x"]}}"#;
        let slice = JsonSlice::new(text);
        let entries: IndexMap<String, RawJson<'_>> =
            bedrock_addon_core::json::parse_at(slice).expect("an object");
        let mut d = Diagnostics::default();
        let at = JsonPath::of_keys(["minecraft:biome", "components"]);
        let map = components(
            &entries,
            slice,
            &at,
            V1_26_30,
            &LoadContext::detached(),
            &mut d,
        );
        let kept: Vec<String> = map
            .iter()
            .map(|entry| match entry {
                ComponentRef::Typed { name, .. } => format!("typed {name}"),
                ComponentRef::Unknown { name, .. } => format!("unknown {name}"),
                _ => "other".to_owned(),
            })
            .collect();
        assert_eq!(kept, ["typed minecraft:tags"]);
        let told: Vec<String> = d
            .iter()
            .map(|f| format!("{:?} {}", f.code, f.location.json_path))
            .collect();
        assert_eq!(
            told,
            ["UnrecognizedComponent { text: None } minecraft:biome/components/demo:nothing"]
        );
    }
}
