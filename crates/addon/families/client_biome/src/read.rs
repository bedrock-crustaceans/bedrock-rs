//! Reading every client-biome struct: the payload object (`description`, `components` one at
//! a time by the components the struct's version binds, and the beta document's
//! `environment_attributes`) and, below 1.21.130, the whole document around it.

use facet::Facet;
use facet_json::RawJson;
use indexmap::IndexMap;

use super::component::{ClientBiomeComponent, child, write_component};
use super::texts::{IDENTIFIER_NAMESPACE, IDENTIFIER_UPPER, INVALID_FILE, fixed};
use super::{Description, EnvironmentAttributes};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, VanillaText,
};
use bedrock_addon_core::document::{ComponentMap, ComponentSet, Extra};
use bedrock_addon_core::json::{EncodeError, JsonSlice, StrictString, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{
    BETA_VERSION, ConvertMode, FieldGate, ParseCx, check_fields_released, parse_payload,
};

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
    fn read_own(key: &str, slice: JsonSlice<'_>, d: &mut Diagnostics) -> Option<Option<Self>> {
        None
    }

    /// The component as JSON, in its era's shape.
    fn write(&self) -> Result<Value, EncodeError>;
}

impl ComponentEra for ClientBiomeComponent {
    fn write(&self) -> Result<Value, EncodeError> {
        write_component(self)
    }
}

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

/// The root of a document below 1.21.130: one strict struct of two members, `format_version`
/// and `minecraft:client_biome`, both required.
#[derive(Facet)]
struct Root<'a> {
    format_version: StrictString,
    #[facet(rename = "minecraft:client_biome")]
    client_biome: RawJson<'a>,
    #[facet(skip, rename = "\u{0}extra")]
    extra: Extra,
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
/// Biome file", and the struct is not built.
pub(crate) fn strict<T>(
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
    read: impl FnOnce(&mut Diagnostics) -> Option<T>,
) -> Option<T> {
    let mut found = Diagnostics::default();
    let parsed = read(&mut found);
    if found.has_errors() {
        found.push(
            Diagnostic::error(DiagnosticCode::MalformedInput {
                message: INVALID_FILE.to_owned(),
                text: Some(VanillaText::Fixed(INVALID_FILE)),
            })
            .at(cx.at.clone()),
        );
    }
    d.extend_with_unknown(found);
    // The chain drops the value of a parse that raised an error.
    parsed
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
        format_version: root.format_version.0,
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
        DiagnosticCode::InvalidValue {
            text: fixed(IDENTIFIER_UPPER),
        }
    } else {
        if namespaced(identifier) {
            return;
        }
        DiagnosticCode::NamespaceRule {
            text: fixed(IDENTIFIER_NAMESPACE),
        }
    };
    d.push(Diagnostic::error(code).at(child(at, &["description", "identifier"])));
}

/// The `components` object of a struct of `version`, in file order. A component the struct
/// binds is read strictly (a failure is an error). A component or member of a later struct is
/// the error "To use … use json format version … or higher" (vanilla: the strict loader's
/// unknown member; the document is not loaded either way). Any other key is an
/// `UnrecognizedComponent`, which the family's strict mask makes an error that refuses the
/// document, as vanilla does for an unknown member.
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
            // The strict mask's input, not a grade: core's `unknown_members_are_errors`
            // (`versioning/pipeline.rs`) turns this Parse-phase warning into the refusing error the
            // game gives for an unknown member.
            d.push(Diagnostic::warn(DiagnosticCode::UnrecognizedComponent { text: None }).at(path));
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
        let mut found = Diagnostics::default();
        let value = match C::read_own(key, slice, &mut found) {
            Some(own) => own,
            None => (meta.parse)(slice, ConvertMode::Strict, cx, &mut found).map(C::from),
        };
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
            Diagnostic::error(DiagnosticCode::InvalidEnumValue { text: None })
                .at(child(at, &["color", "color_map"])),
        );
        reported = true;
    }
    reported
}

/// "To use %s '%s', use json format version %s or higher".
fn use_version(what: &'static str, name: &str, version: FormatVersion) -> DiagnosticCode {
    DiagnosticCode::UnsupportedVersion {
        text: Some(VanillaText::UseFormatVersion {
            what,
            name: name.to_owned(),
            version,
        }),
    }
}
