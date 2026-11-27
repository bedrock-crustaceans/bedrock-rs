//! Reading every client-biome struct: the payload object (`description`, `components` one at
//! a time by the components the struct's version binds, and the beta document's
//! `environment_attributes`) and, below 1.21.130, the whole document around it.

use facet::Facet;

use super::component::ClientBiomeComponent;
use super::history::v1_21_40::{AmbientSounds as OlderAmbientSounds, Component};
use super::{Description, EnvironmentAttributes};
use crate::codes;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Class, Code, Diagnostic, DiagnosticDetail, Diagnostics, Disposition, JsonPath, Kind, Phase,
};
use bedrock_addon_core::document::{ComponentMap, ComponentRef, ComponentSet};
use bedrock_addon_core::id::BiomeId;
use bedrock_addon_core::id::syntax::namespaced_by;
use bedrock_addon_core::json::kit::{Node, Pass, Read};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value, to_json};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::schema::{self, FromJson, Schema};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{
    ConvertMode, FieldGate, ParseCx, check_fields_released, read_payload_with,
};
use bedrock_addon_shared_biome_env::{Whole, read_component, read_row_with, read_struct};

/// The first version read from the `minecraft:client_biome` payload alone (below it the whole
/// document, envelope included, is read as one struct).
pub(crate) const MODERN: FormatVersion = known::V1_21_130;
/// The first struct whose identifier must have a namespace.
pub(crate) const NAMESPACED: FormatVersion = known::V1_21_90;
/// The payload's key.
pub(crate) const PAYLOAD_KEY: &str = "minecraft:client_biome";

/// The members a struct's version does not bind yet, with the first version that does: the
/// strict loader of an older struct rejects them.
const FIELD_GATES: [(&str, &[FieldGate]); 3] = [
    (
        "minecraft:biome_music",
        &[
            FieldGate {
                name: "music_definition",
                released: FormatVersion::new(1, 21, 70),
            },
            FieldGate {
                name: "underwater_music",
                released: known::V1_21_110,
            },
        ],
    ),
    (
        "minecraft:grass_appearance",
        &[FieldGate {
            name: "grass_is_shaded",
            released: known::V1_21_100,
        }],
    ),
    (
        "minecraft:ambient_sounds",
        &[
            FieldGate {
                name: "underwater_addition",
                released: known::V1_21_120,
            },
            FieldGate {
                name: "underwater_loop",
                released: known::V1_21_120,
            },
            FieldGate {
                name: "underwater_mood",
                released: known::V1_21_120,
            },
        ],
    ),
];
/// The first struct whose component `minecraft:ambient_sounds` has the underwater members: below
/// it the component is read in the shape its era had.
const AMBIENT_SOUNDS_NEWEST: FormatVersion = known::V1_21_120;
/// The key of `minecraft:ambient_sounds`.
const AMBIENT_SOUNDS: &str = "minecraft:ambient_sounds";

/// The component set of one era of the chain: the newest set, or an older one that reads a
/// component in the shape its era had.
pub(crate) trait ComponentEra: ComponentSet + From<ClientBiomeComponent> {
    /// The components of a payload read as the older set's, as this era's.
    fn from_read(components: Components) -> ComponentMap<Self>;

    /// The component as JSON, in its era's shape.
    fn write(&self) -> Result<Value, EncodeError>;
}

impl ComponentEra for ClientBiomeComponent {
    /// Every component of the newest set's eras is read in the newest shape.
    fn from_read(components: Components) -> ComponentMap<Self> {
        let mut map = ComponentMap::new();
        for entry in components.0.iter() {
            match entry {
                ComponentRef::Typed {
                    value: Component::Newest(component),
                    synthesized,
                    ..
                } => {
                    map.insert_typed(component.clone(), synthesized);
                }
                ComponentRef::Unknown { name, json } => {
                    map.insert_unknown(name, json.clone());
                }
                _ => {}
            }
        }
        map
    }

    fn write(&self) -> Result<Value, EncodeError> {
        self.write_component()
    }
}

/// What a payload object holds: the components are those of the struct the read is of
/// ([`read_as`]), in the older set.
#[derive(Facet, Schema)]
pub(crate) struct Payload {
    pub(crate) description: Description,
    #[facet(opaque, schema::read_with = components)]
    pub(crate) components: Components,
}

/// The beta payload: [`Payload`] and the environment attributes.
#[derive(Facet, Schema)]
pub(crate) struct BetaPayload {
    #[facet(flatten)]
    pub(crate) payload: Payload,
    pub(crate) environment_attributes: Option<EnvironmentAttributes>,
}

/// The root of a document below 1.21.130: one strict struct of two members, `format_version`
/// and `minecraft:client_biome`, both required.
#[derive(Facet, Schema)]
pub(crate) struct Root {
    pub(crate) format_version: String,
    #[facet(rename = "minecraft:client_biome", schema::read_with = read_struct::<Payload>)]
    pub(crate) client_biome: Payload,
}

/// The components of a payload.
pub(crate) struct Components(pub(crate) ComponentMap<Component>);

// The components are read by the family's own loop and hold no Molang walk of their own here
// (the struct that owns them walks them), no bound and no trigger.
bedrock_addon_core::impl_no_molang!(Components);
bedrock_addon_core::impl_schema_leaf!(Components);

/// Reads the payload `T` of the struct of `version`: its components and identifier are the
/// ones `version` binds.
pub(crate) fn read_as<T: FromJson>(
    slice: JsonSlice<'_>,
    version: FormatVersion,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<T> {
    let whole: Option<Whole<T>> = read_payload_with(slice, cx, d, |read| {
        crate::typed_read(read.format(Some(version)))
    });
    whole.map(|whole| whole.0)
}

/// Runs a struct's strict read; any error it raises is followed by vanilla's "Invalid Client
/// Biome file", and the struct is not built. An unknown member, which the slice's mask grades
/// an error, is not counted: the line follows the other failures alone.
pub(crate) fn strict<T>(
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
    read: impl FnOnce(&mut Diagnostics) -> Option<T>,
) -> Option<T> {
    let mut found = Diagnostics::default();
    let parsed = read(&mut found);
    if found
        .iter()
        .any(|finding| finding.rejects() && !is_unknown_member(finding))
    {
        found.push(
            Diagnostic::of(&codes::INVALID_FILE)
                .detail(DiagnosticDetail::Syntax(
                    codes::INVALID_FILE.template().to_owned(),
                ))
                .at(cx.at.clone()),
        );
    }
    d.extend_with_unknown(found);
    // The chain drops the value of a parse that raised an error.
    parsed
}

/// Whether `finding` tells of an unknown member or component.
pub(crate) fn is_unknown_member(finding: &Diagnostic) -> bool {
    // The kind holds the unrecognized field and component definitions and no others.
    matches!(finding.kind(), Kind::UnknownMember(Disposition::Kept))
}

/// A character of `[a-z0-9._%+-]`.
fn name_byte(byte: u8) -> bool {
    byte.is_ascii_lowercase()
        || byte.is_ascii_digit()
        || matches!(byte, b'.' | b'_' | b'%' | b'+' | b'-')
}

/// `^[a-z0-9._%+-]+:[a-z0-9._%+-]+$`, the identifier's regex from 1.21.90.
fn namespaced(identifier: &str) -> bool {
    namespaced_by(identifier, name_byte)
}

/// `^[a-z0-9._%+-:]+$`, the identifier's regex below 1.21.90. In the class, `+-:` is the range
/// from `+` to `:`, so `,` and `/` match too.
fn bare_or_namespaced(identifier: &str) -> bool {
    !identifier.is_empty()
        && identifier
            .bytes()
            .all(|b| name_byte(b) || (b'+'..=b':').contains(&b))
}

/// The description's `identifier` (`schema::read_with`) under the identifier's regex constraint
/// of the struct of the read's version.
pub(crate) fn identifier(node: &Node<'_>, read: &mut Read<'_>) -> Option<BiomeId> {
    let identifier = BiomeId::from_json(node, read)?;
    let version = read.known_format()?;
    let text = identifier.as_str();
    let code = if version < NAMESPACED {
        (!bare_or_namespaced(text)).then_some(Code::from(&codes::IDENTIFIER_UPPER))
    } else {
        (!namespaced(text)).then_some(Code::from(&codes::IDENTIFIER_NAMESPACE))
    };
    if let Some(code) = code {
        read.defer(
            Pass::Misread,
            Diagnostic::error(code).at(node.path().clone()),
        );
    }
    Some(identifier)
}

/// The `components` object of a struct of the read's version, in file order
/// (`schema::read_with`). A component the struct binds is read strictly (a failure is an
/// error). A component or member of a later struct is the error "To use … use json format
/// version … or higher" (vanilla: the strict loader's unknown member; the document is not
/// loaded either way). Any other key is an `UnrecognizedComponent`, graded by the slice's
/// reading: the family's strict mask makes it an error that refuses the document, as vanilla
/// does for an unknown member.
fn components(node: &Node<'_>, read: &mut Read<'_>) -> Option<Components> {
    let version = read.known_format()?;
    let table = ClientBiomeComponent::TABLE;
    let members = read.object(node)?;
    let mut cx = read
        .load_context()
        .cloned()
        .unwrap_or_else(LoadContext::detached);
    if read.load_context().is_none() {
        cx.reading = read.reading();
    }
    cx.document_format = Some(version);
    let mut map = ComponentMap::new();
    let mut found = Diagnostics::default();
    for (key, member) in members.iter() {
        let path = node.path().child(key);
        let Some(meta) = table.get(key) else {
            map.insert_unknown(key, member.value().clone());
            found.push(
                Diagnostic::of(&core_codes::UNRECOGNIZED_COMPONENT)
                    .at(path)
                    .severity(cx.grade(Class::Unknown)),
            );
            continue;
        };
        // Here, not on the finished map: a component the struct of `version` does not bind is
        // never read into one.
        if let Some(released) = meta.released.filter(|released| version < *released) {
            found.push(Diagnostic::error(use_version("component", key, released)).at(path));
            continue;
        }
        if gated(key, member.value(), version, &path, &mut found) {
            continue;
        }
        // A member without a text (a node made without one) is read from its value written out.
        let written;
        let slice = if let Some(slice) = member.text() {
            slice
        } else {
            written = to_json(member.value()).unwrap_or_default();
            JsonSlice::new(&written)
        };
        let value = read_component(&path, &mut found, |at| {
            if key == AMBIENT_SOUNDS && version < AMBIENT_SOUNDS_NEWEST {
                read_row_with::<OlderAmbientSounds, Component>(
                    slice,
                    ConvertMode::Strict,
                    &cx,
                    at,
                    crate::typed_read,
                )
            } else {
                (meta.parse)(slice, ConvertMode::Strict, &cx, at).map(Component::from)
            }
        });
        if let Some(value) = value {
            map.insert_typed(value, false);
        }
    }
    let kept = found.take_unknown();
    // Told with the passes after the payload's own members, as a struct that reads its payload
    // first and its components next tells them.
    for finding in found {
        read.defer(Pass::Misread, finding);
    }
    read.findings_mut()
        .unknown_mut()
        .absorb(&JsonPath::new(), kept);
    Some(Components(map))
}

/// The members of the component `key` that the struct of `version` does not bind yet; `true`
/// when one was reported.
fn gated(
    key: &str,
    json: &Value,
    version: FormatVersion,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> bool {
    let Some(object) = json.as_object() else {
        return false;
    };
    let mut reported = false;
    // Core's format-version field gate makes the findings; they are raised while
    // the struct is read, so their phase is the parse's.
    for (_, fields) in FIELD_GATES
        .iter()
        .filter(|(component, _)| *component == key)
    {
        let mut found = Diagnostics::default();
        reported |= !check_fields_released(
            fields,
            |member| object.get(member).is_some(),
            key.trim_start_matches("minecraft:"),
            version,
            at,
            &mut found,
        );
        d.extend(found.into_iter().map(|mut finding| {
            finding.phase = Phase::Parse;
            finding
        }));
    }
    reported
}

/// "To use %s '%s', use json format version %s or higher".
fn use_version(what: &'static str, name: &str, version: FormatVersion) -> Code {
    core_codes::USE_FORMAT_VERSION.with([what.to_string(), name.to_string(), version.to_string()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::load::LoadContext;

    /// An unknown component key is told and kept in the map as an unknown entry (biome drops
    /// it); the strict mask refuses the document either way, so only this read shows the
    /// difference.
    #[test]
    fn an_unknown_component_key_is_kept_as_an_unknown_entry() {
        let text = r#"{"demo:nothing": {"a": 1}}"#;
        let parsed = bedrock_addon_core::json::parse::parse(
            text,
            bedrock_addon_core::json::parse::Options::default(),
        )
        .expect("an object");
        let at = JsonPath::of_keys(["minecraft:client_biome", "components"]);
        let load = LoadContext::detached();
        let mut d = Diagnostics::default();
        let map = {
            let mut read = Read::new(load.reading, &mut d)
                .load(&load)
                .format(Some(MODERN));
            components(&Node::parsed(&parsed, at), &mut read).expect("an object")
        };
        let kept: Vec<String> = map
            .0
            .iter()
            .map(|entry| match entry {
                ComponentRef::Typed { name, .. } => format!("typed {name}"),
                ComponentRef::Unknown { name, json } => format!("unknown {name} {json:?}"),
                _ => "other".to_owned(),
            })
            .collect();
        assert_eq!(kept.len(), 1);
        assert!(kept[0].starts_with("unknown demo:nothing"), "{kept:?}");
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
            ["core/unrecognized_component minecraft:client_biome/components/demo:nothing"]
        );
    }

    /// The members no struct binds are kept at every depth of a payload read, where the load
    /// refuses the document for them (the strict slices): the table is the read's, not the
    /// load's. One older ambient-sounds struct beside the newest.
    #[test]
    fn a_payload_read_keeps_its_unknown_members_at_every_depth() {
        use bedrock_addon_core::version::FormatVersionInput;
        let text = r##"{"zz": 1, "description": {"identifier": "demo:a", "zz": 2}, "components": {"minecraft:sky_color": {"sky_color": "#ffffff", "zz": 3}, "minecraft:biome_music": {"volume_multiplier": 0.5, "zz": 4}, "minecraft:ambient_sounds": {"addition": {"asset": "demo.sound", "chance": 0.5, "zz": 5}, "zz": 6}, "minecraft:precipitation": {"ash": 0.5, "zz": 7}, "minecraft:water_appearance": {"surface_color": "#ffffff", "zz": 8}}}"##;
        let load = LoadContext::detached();
        let cx = ParseCx {
            load: &load,
            written: FormatVersionInput {
                version: Some(MODERN),
                beta_literal: false,
                prerelease: false,
            },
            original_version: MODERN,
            at: JsonPath::of_keys(["minecraft:client_biome"]),
        };
        let mut d = Diagnostics::default();
        read_as::<Payload>(JsonSlice::new(text), MODERN, &cx, &mut d).expect("a payload");
        let kept: Vec<String> = d
            .unknown()
            .iter()
            .map(|(at, key, _)| format!("{at}/{key}"))
            .collect();
        assert_eq!(
            kept,
            [
                "minecraft:client_biome/zz",
                "minecraft:client_biome/components/minecraft:ambient_sounds/zz",
                "minecraft:client_biome/components/minecraft:ambient_sounds/addition/zz",
                "minecraft:client_biome/components/minecraft:biome_music/zz",
                "minecraft:client_biome/components/minecraft:precipitation/zz",
                "minecraft:client_biome/components/minecraft:sky_color/zz",
                "minecraft:client_biome/components/minecraft:water_appearance/zz",
                "minecraft:client_biome/description/zz",
            ]
        );
    }

    /// The same of the older ambient sounds, read by their own struct below 1.21.120.
    #[test]
    fn an_older_ambient_sounds_read_keeps_its_unknown_members() {
        use bedrock_addon_core::version::FormatVersionInput;
        let text = r#"{"description": {"identifier": "demo:a"}, "components": {"minecraft:ambient_sounds": {"addition": "demo.sound", "zz": 1}}}"#;
        let load = LoadContext::detached();
        let version = known::V1_21_90;
        let cx = ParseCx {
            load: &load,
            written: FormatVersionInput {
                version: Some(version),
                beta_literal: false,
                prerelease: false,
            },
            original_version: version,
            at: JsonPath::of_keys(["minecraft:client_biome"]),
        };
        let mut d = Diagnostics::default();
        read_as::<Payload>(JsonSlice::new(text), version, &cx, &mut d).expect("a payload");
        let kept: Vec<String> = d
            .unknown()
            .iter()
            .map(|(at, key, _)| format!("{at}/{key}"))
            .collect();
        assert_eq!(
            kept,
            ["minecraft:client_biome/components/minecraft:ambient_sounds/zz"]
        );
    }
}
