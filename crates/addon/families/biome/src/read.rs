//! Reading the payload object of every biome struct: `description`, `components` (one at a
//! time, by the components the struct's version binds) and, in the beta document,
//! `environment_attributes`. The payload is a derived struct read by `read_payload_with`; its
//! `components` member is the one hand-written read, the loop over the keys the struct binds.

use facet::Facet;

use super::component::BiomeComponent;
use super::{BiomeFamily, Description, EnvironmentAttributes};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Class, Diagnostic, Diagnostics};
use bedrock_addon_core::document::ComponentMap;
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::json::kit::{Node, Read, Versions};
use bedrock_addon_core::schema::{self, FromJson, Schema};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{
    ConvertMode, KeyGate, ParseCx, read_component_map_by, read_payload_with,
};
use bedrock_addon_shared_biome_env::{Whole, read_struct};

/// The first version read from the `minecraft:biome` payload alone (below it the whole
/// document, envelope included, is read as one struct).
pub(crate) const CEREAL: FormatVersion = known::V1_21_120;
/// The first struct with `minecraft:village_type` and the nether replacement.
pub(crate) const V1_26_0: FormatVersion = known::V1_26_0;
/// The first struct with `minecraft:subsurface_builder` and the noise-gradient builder.
pub(crate) const V1_26_30: FormatVersion = known::V1_26_30;

/// What a payload object holds: the components are those of the struct the read is of
/// ([`read_as`]).
#[derive(Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub(crate) struct Payload {
    pub(crate) description: Description,
    #[facet(opaque, schema::read_with = components)]
    pub(crate) components: Components,
}

/// The beta payload: [`Payload`] and the environment attributes.
#[derive(Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub(crate) struct BetaPayload {
    #[facet(flatten)]
    pub(crate) payload: Payload,
    pub(crate) environment_attributes: Option<EnvironmentAttributes>,
}

/// The root of a document below 1.21.120: one strict struct of two members.
#[derive(Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub(crate) struct Root {
    pub(crate) format_version: String,
    #[facet(rename = "minecraft:biome", schema::read_with = read_struct::<Payload>)]
    pub(crate) biome: Payload,
}

/// The components of a payload.
pub(crate) struct Components(pub(crate) ComponentMap<BiomeComponent>);

// The components are read by the family's own loop and hold no Molang walk of their own here
// (the struct that owns them walks them), no bound and no trigger.
bedrock_addon_core::impl_no_molang!(Components);
bedrock_addon_core::impl_schema_leaf!(Components);

/// Reads the payload `T` of the struct of `version`: its components are the ones `version`
/// binds.
pub(crate) fn read_as<T: FromJson>(
    slice: JsonSlice<'_>,
    version: FormatVersion,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<T> {
    let whole: Option<Whole<T>> = read_payload_with(slice, cx, d, |read| {
        let engine = read.known_versions().engine;
        read.versions(Versions {
            format: Some(version),
            engine,
        })
    });
    whole.map(|whole| whole.0)
}

/// The `components` object of a struct of the read's version, in file order
/// (`schema::read_with`). A component the struct binds is read strictly (a failure is an
/// error). Any other key, a component of a later struct (`minecraft:village_type` before
/// 1.26.0, `minecraft:subsurface_builder` before 1.26.30) included, is an
/// `UnrecognizedComponent` and dropped, graded by the slice's reading: the family's strict mask
/// makes it an error that refuses the document, as vanilla does for an unknown member in every
/// format version.
fn components(node: &Node<'_>, read: &mut Read<'_>) -> Option<Components> {
    let version = read.known_versions().format?;
    let table = BiomeComponent::TABLE;
    let severity = read.reading().grade(Class::Unknown);
    read_component_map_by::<BiomeFamily>(
        node,
        read,
        ConvertMode::Strict,
        |_| false,
        |key| match table.get(key) {
            Some(meta) if meta.released.is_some_and(|released| version >= released) => {
                KeyGate::Read
            }
            _ => KeyGate::Drop(
                Diagnostic::of(&core_codes::UNRECOGNIZED_COMPONENT).severity(severity),
            ),
        },
    )
    .map(Components)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::diagnostic::JsonPath;
    use bedrock_addon_core::document::ComponentRef;
    use bedrock_addon_core::load::LoadContext;

    /// An unknown component key is told and left out of the map (`client_biome` keeps it as an
    /// unknown entry); the strict mask refuses the document either way, so only this read
    /// shows the difference.
    #[test]
    fn an_unknown_component_key_is_dropped_from_the_map() {
        let text = r#"{"demo:nothing": {"a": 1}, "minecraft:tags": {"tags": ["x"]}}"#;
        let parsed = bedrock_addon_core::json::parse::parse(
            text,
            bedrock_addon_core::json::parse::Options::default(),
        )
        .expect("an object");
        let at = JsonPath::of_keys(["minecraft:biome", "components"]);
        let load = LoadContext::detached();
        let mut d = Diagnostics::default();
        let map = {
            let mut read = Read::new(load.reading, &mut d)
                .load(&load)
                .versions(Versions {
                    format: Some(V1_26_30),
                    engine: None,
                });
            components(&Node::parsed(&parsed, at), &mut read).expect("an object")
        };
        let kept: Vec<String> = map
            .0
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
            .map(|f| {
                format!(
                    "{} {}",
                    bedrock_addon_core::diagnostic::testing::shown(f),
                    f.location.json_path
                )
            })
            .collect();
        assert_eq!(
            told,
            ["core/unrecognized_component minecraft:biome/components/demo:nothing"]
        );
    }
}
