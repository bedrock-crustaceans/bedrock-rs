//! Reading every client-biome struct: the payload object (`description`, `components` one at
//! a time by the components the struct's version binds, and the beta document's
//! `environment_attributes`) and, below 1.21.130, the whole document around it.

use facet::Facet;
use facet_json::RawJson;
use indexmap::IndexMap;

use super::component::{ClientBiomeComponent, child};
use super::{Description, EnvironmentAttributes};
use crate::codes;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Class, Code, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, Phase,
};
use bedrock_addon_core::document::{ComponentMap, ComponentSet};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::schema;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{
    BETA_VERSION, ConvertMode, FieldGate, ParseCx, check_fields_released, parse_payload,
};
use bedrock_addon_shared_biome_env::read_component;

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
/// The first struct whose foliage colour-map enum has `dry_foliage`.
const DRY_FOLIAGE_MAP: FormatVersion = FormatVersion::new(1, 21, 70);

/// The component set of one era of the chain: the newest set, or an older one that reads a
/// component in the shape its era had.
pub(crate) trait ComponentEra: ComponentSet + From<ClientBiomeComponent> {
    /// Reads the component `key` when its shape in this era is not the newest one's:
    /// `None` when the newest set reads it, else the component or its failure (reported in
    /// `d`, relative to the component).
    #[allow(unused_variables)]
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
        None
    }

    /// The component as JSON, in its era's shape.
    fn write(&self) -> Result<Value, EncodeError>;
}

impl ComponentEra for ClientBiomeComponent {
    fn write(&self) -> Result<Value, EncodeError> {
        self.write_component()
    }
}

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

/// The root of a document below 1.21.130: one strict struct of two members, `format_version`
/// and `minecraft:client_biome`, both required.
#[derive(Facet)]
#[facet(schema::unknown = "keep")]
struct Root<'a> {
    format_version: String,
    #[facet(rename = "minecraft:client_biome")]
    client_biome: RawJson<'a>,
}

/// What a payload object holds.
pub(crate) struct Payload<C: ComponentSet> {
    pub(crate) description: Description,
    pub(crate) components: ComponentMap<C>,
}

/// A document below 1.21.130: its `format_version` as written and its payload.
pub(crate) struct Whole<C: ComponentSet> {
    pub(crate) format_version: String,
    pub(crate) payload: Payload<C>,
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
    // By legacy name: the variant and the definitions that were it.
    matches!(
        finding.code.name(),
        "UnrecognizedField" | "UnrecognizedComponent"
    )
}

/// Reads a stable payload object as the struct of `version` binds it.
pub(crate) fn payload<C: ComponentEra>(
    slice: JsonSlice<'_>,
    version: FormatVersion,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<Payload<C>> {
    let shell: Shell<'_> = parse_payload(slice, cx, d)?;
    check_identifier(&shell.description, version, &cx.at, d);
    let at = child(&cx.at, &["components"]);
    let components = components(&shell.components, slice, &at, version, cx.load, d);
    Some(Payload {
        description: shell.description,
        components,
    })
}

/// Reads the beta payload object: the 1.21.130 components and `environment_attributes`.
pub(crate) fn beta_payload(
    slice: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<(Payload<ClientBiomeComponent>, Option<EnvironmentAttributes>)> {
    let shell: BetaShell<'_> = parse_payload(slice, cx, d)?;
    check_identifier(&shell.description, BETA_VERSION, &cx.at, d);
    let at = child(&cx.at, &["components"]);
    let components = components(&shell.components, slice, &at, BETA_VERSION, cx.load, d);
    let payload = Payload {
        description: shell.description,
        components,
    };
    Some((payload, shell.environment_attributes))
}

/// Reads a document below 1.21.130, root first.
pub(crate) fn whole<C: ComponentEra>(
    document: JsonSlice<'_>,
    version: FormatVersion,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<Whole<C>> {
    let root: Root<'_> = parse_payload(document, cx, d)?;
    let slice = document.nested(&root.client_biome)?;
    let inner = ParseCx {
        at: child(&cx.at, &[PAYLOAD_KEY]),
        ..cx.clone()
    };
    let payload = payload(slice, version, &inner, d)?;
    Some(Whole {
        format_version: root.format_version,
        payload,
    })
}

/// A character of `[a-z0-9._%+-]`.
fn name_byte(byte: u8) -> bool {
    byte.is_ascii_lowercase()
        || byte.is_ascii_digit()
        || matches!(byte, b'.' | b'_' | b'%' | b'+' | b'-')
}

/// `^[a-z0-9._%+-]+:[a-z0-9._%+-]+$`, the identifier's regex from 1.21.90.
fn namespaced(identifier: &str) -> bool {
    let part = |text: &str| !text.is_empty() && text.bytes().all(name_byte);
    identifier
        .split_once(':')
        .is_some_and(|(namespace, name)| part(namespace) && part(name))
}

/// `^[a-z0-9._%+-:]+$`, the identifier's regex below 1.21.90. In the class, `+-:` is the range
/// from `+` to `:`, so `,` and `/` match too.
fn bare_or_namespaced(identifier: &str) -> bool {
    !identifier.is_empty()
        && identifier
            .bytes()
            .all(|b| name_byte(b) || (b'+'..=b':').contains(&b))
}

/// The identifier's regex constraint of the struct of `version`.
fn check_identifier(
    description: &Description,
    version: FormatVersion,
    at: &JsonPath,
    d: &mut Diagnostics,
) {
    let identifier = description.identifier.as_str();
    let code = if version < NAMESPACED {
        if bare_or_namespaced(identifier) {
            return;
        }
        Code::from(&codes::IDENTIFIER_UPPER)
    } else {
        if namespaced(identifier) {
            return;
        }
        Code::from(&codes::IDENTIFIER_NAMESPACE)
    };
    d.push(Diagnostic::error(code).at(child(at, &["description", "identifier"])));
}

/// The `components` object of a struct of `version`, in file order. A component the struct
/// binds is read strictly (a failure is an error). A component or member of a later struct is
/// the error "To use … use json format version … or higher" (vanilla: the strict loader's
/// unknown member; the document is not loaded either way). Any other key is an
/// `UnrecognizedComponent`, graded by the slice's reading: the family's strict mask makes it an
/// error that refuses the document, as vanilla does for an unknown member.
fn components<C: ComponentEra>(
    entries: &IndexMap<String, RawJson<'_>>,
    within: JsonSlice<'_>,
    at: &JsonPath,
    version: FormatVersion,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> ComponentMap<C> {
    let table = ClientBiomeComponent::TABLE;
    let mut map = ComponentMap::new();
    for (key, raw) in entries {
        let slice = within.nested(raw).unwrap_or(JsonSlice {
            text: raw.as_str(),
            offset: within.offset,
        });
        let path = child(at, &[key]);
        let json =
            bedrock_addon_core::json::from_str_jsonc::<Value>(slice.text).unwrap_or(Value::NULL);
        let Some(meta) = table.get(key) else {
            map.insert_unknown(key, json);
            d.push(
                Diagnostic::of(&core_codes::UNRECOGNIZED_COMPONENT)
                    .at(path)
                    .severity(cx.grade(Class::Unknown)),
            );
            continue;
        };
        // Here, not on the finished map: a component the struct of `version` does not bind is
        // never read into one.
        if let Some(released) = meta.released.filter(|released| version < *released) {
            d.push(Diagnostic::error(use_version("component", key, released)).at(path));
            continue;
        }
        if gated(key, &json, version, &path, d) {
            continue;
        }
        let value = read_component(&path, d, |found| match C::read_own(key, slice, cx, found) {
            Some(own) => own,
            None => (meta.parse)(slice, ConvertMode::Strict, cx, found).map(C::from),
        });
        if let Some(value) = value {
            map.insert_typed(value, false);
        }
    }
    map
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
    let dry_foliage = key == "minecraft:foliage_appearance"
        && object
            .get("color")
            .and_then(Value::as_object)
            .and_then(|color| color.get("color_map"))
            .and_then(Value::as_string)
            .is_some_and(|name| name.as_str() == "dry_foliage");
    if dry_foliage && version < DRY_FOLIAGE_MAP {
        d.push(
            Diagnostic::of(&core_codes::INVALID_ENUM_VALUE).at(child(at, &["color", "color_map"])),
        );
        reported = true;
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
    use bedrock_addon_core::document::ComponentRef;

    /// An unknown component key is told and kept in the map as an unknown entry (biome drops
    /// it); the strict mask refuses the document either way, so only this read shows the
    /// difference.
    #[test]
    fn an_unknown_component_key_is_kept_as_an_unknown_entry() {
        let text = r#"{"demo:nothing": {"a": 1}}"#;
        let slice = JsonSlice::new(text);
        let entries: IndexMap<String, RawJson<'_>> =
            bedrock_addon_core::json::parse_at(slice).expect("an object");
        let mut d = Diagnostics::default();
        let at = JsonPath::of_keys(["minecraft:client_biome", "components"]);
        let map: ComponentMap<ClientBiomeComponent> = components(
            &entries,
            slice,
            &at,
            MODERN,
            &LoadContext::detached(),
            &mut d,
        );
        let kept: Vec<String> = map
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
                    bedrock_addon_core::diagnostic::testing::legacy_debug(f),
                    f.location.json_path
                )
            })
            .collect();
        assert_eq!(
            told,
            ["UnrecognizedComponent { text: None } minecraft:client_biome/components/demo:nothing"]
        );
    }
}
