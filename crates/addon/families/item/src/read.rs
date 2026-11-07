//! Reading items from their text: the routing between the legacy and the component formats,
//! the load of one file (the resource-pack rule, the chain, the JSON item path's texts) and the
//! modern formats' payload.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::document::{Document, Origin, Rejected};
use bedrock_addon_core::id::ItemId;
use bedrock_addon_core::json::{JsonSlice, PrepareError, Prepared, Value};
use bedrock_addon_core::load::{DocumentSource, LoadContext};
use bedrock_addon_core::pack_identity::PackType;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, known};
use bedrock_addon_core::versioning::{
    self, DocumentRead, Family, LoadOutcome, Loaded, OpenBeta, ParseCx,
};
use bedrock_addon_shared_shapes::CreativeCategory;
use facet_json::RawJson;
use indexmap::IndexMap;

use super::component::ItemComponent;
use super::history::{
    HISTORY_ROWS, ItemFamily, into_component_map, parse_parts, written_components,
};
use super::types::{ModernItem, modern_version};
use super::{ItemDescription, codes};

/// 1.16.100: the first component format, and the start of vanilla's data-driven items.
pub(crate) const V1_16_100: FormatVersion = known::V1_16_100;

/// Why a document is in the legacy item format: the version (a) or the components (b) fail
/// the game's check that a schema is component based.
pub(crate) fn is_component_based(
    payload: JsonSlice<'_>,
    v: FormatVersionInput,
) -> Result<(), &'static str> {
    let version_ok = match v.version {
        None => false,
        Some(version) => {
            (version == V1_16_100 && !v.prerelease) || (version.major == 1 && version.minor >= 17)
        }
    };
    if !version_ok {
        return Err("the format version is neither 1.16.100 nor 1.17.0 or later");
    }
    let probe = bedrock_addon_core::json::from_str_borrowed_jsonc::<Probe<'_>>(payload.text).ok();
    let registered = probe
        .and_then(|p| p.components)
        .is_some_and(|components| components.keys().any(|key| is_registered(key)));
    if registered {
        Ok(())
    } else {
        Err("`components` names no registered component and no `tag:` key")
    }
}

/// What the routing reads of a payload: its component names.
#[derive(facet::Facet)]
struct Probe<'a> {
    components: Option<IndexMap<String, RawJson<'a>>>,
}

/// Whether vanilla's component factory registers `key`, or it is a `tag:` key.
pub(crate) fn is_registered(key: &str) -> bool {
    super::legacy::is_tag_key(key)
        || ItemComponent::TABLE.get(key).is_some()
        || HISTORY_ROWS.iter().any(|row| row.name == key)
}

/// What one item file loads to.
pub(crate) struct ItemLoad {
    /// The outcome of the chain.
    pub(crate) outcome: LoadOutcome<ItemFamily>,
    /// The identifier the payload names, when it names one.
    pub(crate) identifier: Option<String>,
}

/// The text of an item file, ready for the envelope: its BOM blanked, UTF-8 (else
/// `MalformedInput`), nested at most `MAX_JSON_DEPTH` deep (else `MalformedInput`).
/// A root `use_beta_features` is no member of the current format: like any other root member
/// beside the version and the payload it is read by nothing (the item envelope does not
/// count members).
pub(crate) fn prepare(bytes: &[u8]) -> Result<Prepared<'_>, Diagnostics> {
    Prepared::new(bytes).map_err(|error| {
        let message = match error {
            PrepareError::TooDeep(_) => "nested too deeply",
            PrepareError::NotUtf8(_) | PrepareError::Empty => "not UTF-8",
        };
        let mut early = Diagnostics::default();
        early.push(Diagnostic::of(
            bedrock_addon_core::diagnostic::builders::syntax(message.to_owned()),
        ));
        early
    })
}

/// Loads one item file: the resource-pack rule, then the chain.
pub(crate) fn load_item(bytes: &[u8], cx: &LoadContext) -> ItemLoad {
    // A leading byte-order mark is blanked, not cut off, so every span is an offset into the
    // file as written.
    let prepared = match prepare(bytes) {
        Ok(prepared) => prepared,
        Err(early) => return rejected_load(early),
    };
    let text = prepared.as_str();
    let document = JsonSlice::new(text);
    let mut parsed = versioning::parse_as_written::<ItemFamily>(document, cx);
    parsed.diagnostics.extend(numeric_version(text));
    json_path_texts(&mut parsed.diagnostics, text, cx);
    let identifier = parsed.identity.identifier.clone();
    // a resource-pack item at or above 1.16.100
    if cx.pack_type == PackType::Resource
        && parsed.version.effective >= V1_16_100
        && !parsed.version.beta
    {
        let mut diagnostics = parsed.diagnostics;
        diagnostics.push(Diagnostic::error(
            codes::ITEM_IN_RESOURCE_PACK
                .with([identifier.clone().unwrap_or_default()])
                .into(),
        ));
        return ItemLoad {
            outcome: LoadOutcome {
                value: None,
                original_version: parsed.version.effective,
                version: parsed.version,
                diagnostics,
            },
            identifier,
        };
    }
    let outcome = versioning::finish::<ItemFamily>(parsed, cx);
    ItemLoad {
        outcome,
        identifier,
    }
}

/// The texts of the JSON item path (the game's JSON upgrade step) in place of the framework's
/// envelope texts, each an error that refuses the item: a JSON syntax error gets the "invalid
/// json object" line alone; a root that is no object gets the "root value needs to be a json
/// object" line and then the invalid object line; a version not found or not readable gets
/// its own line and then the invalid object line; a version above the game's current one
/// gets the "out of handled range" line; no `minecraft:item` object gets the "should be a
/// json object" line. An MCB document skips the JSON upgrade step and keeps the framework's
/// validation texts.
pub(crate) fn json_path_texts(d: &mut Diagnostics, text: &str, cx: &LoadContext) {
    if cx.source == DocumentSource::Mcb {
        return;
    }
    let root = bedrock_addon_core::json::from_str_jsonc::<Value>(text).ok();
    let payload_not_object = root
        .as_ref()
        .and_then(Value::as_object)
        .and_then(|root| root.get(ItemFamily::PAYLOAD_KEY))
        .is_some_and(|payload| !payload.is_object());
    let payload_at = JsonPath::new().child(ItemFamily::PAYLOAD_KEY);
    let mut invalid_object = false;
    for finding in d.iter_mut() {
        if finding.severity < Severity::Error {
            continue;
        }
        // The JSON reader's refusal, told again in the item's words; the reader's message
        // stays the detail.
        if finding.is(&core_codes::DOCUMENT_JSON) {
            let message = finding.syntax_message().unwrap_or_default().to_owned();
            let line = if root.is_some() {
                invalid_object = true;
                &codes::ROOT_NOT_OBJECT
            } else {
                &codes::INVALID_JSON_OBJECT
            };
            finding.code = line.into();
            finding.detail = Some(Box::new(DiagnosticDetail::Syntax(message)));
            continue;
        }
        // the versioned parser's "no parser available for version", and its version
        let no_parser = match &finding.code {
            DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::NoParserForVersion { version }),
            } => Some(version.as_str()),
            DiagnosticCode::Specific { code } if code.is(&core_codes::NO_PARSER_FOR_VERSION) => {
                code.args().first().map(String::as_str)
            }
            _ => None,
        };
        let out_of_range = no_parser.is_some_and(|version| version != "beta");
        // By legacy name: the variant and the definitions that were it.
        match finding.code.name() {
            "MalformedFormatVersion" => {
                finding.code = (&codes::VERSION_NOT_READABLE).into();
                invalid_object = true;
            }
            "MissingFormatVersion" => invalid_object = true,
            "MissingPayload" => {
                finding.code = (&codes::PAYLOAD_NOT_OBJECT).into();
            }
            _ if out_of_range => {
                finding.code = (&codes::VERSION_OUT_OF_RANGE).into();
            }
            _ if no_parser.is_some() => {}
            _ if payload_not_object && finding.location.json_path == payload_at => {
                finding.code = (&codes::PAYLOAD_NOT_OBJECT_MALFORMED_INPUT).into();
                finding.detail = Some(Box::new(DiagnosticDetail::Syntax(
                    "the payload is no object".to_owned(),
                )));
            }
            _ => {}
        }
    }
    if invalid_object {
        d.push(
            Diagnostic::error((&codes::INVALID_JSON_OBJECT).into()).detail(
                DiagnosticDetail::Syntax("the item is not loaded".to_owned()),
            ),
        );
    }
}

/// A root `format_version` written as a number (the crate's own text). The game re-reads
/// the version of a JSON item from its text when it is no string.
const NUMERIC_VERSION: &str =
    "the version written as a number is read from its text; it is written back as a string";

/// A root `format_version` written as a number: the item's version is read from its text, as
/// the game's document parse re-reads a version that is no string, and the writer writes a
/// string. Told where it is, an information.
pub(crate) fn numeric_version(text: &str) -> Option<Diagnostic> {
    let root = bedrock_addon_core::json::from_str_jsonc::<Value>(text).ok()?;
    let version = root.as_object()?.get("format_version")?;
    version.as_number()?;
    let at = JsonPath::new().child("format_version");
    Some(Diagnostic::of(&core_codes::VALUE_REWRITTEN).at(at).detail(
        bedrock_addon_core::diagnostic::DiagnosticDetail::Note(NUMERIC_VERSION),
    ))
}

/// The identifier of the item file `bytes` of the pack of `cx`, when the file loads there:
/// what the link pass reads of a resource pack's `items/` files, which no collection holds.
pub fn loaded_identifier(bytes: &[u8], cx: &LoadContext) -> Option<String> {
    let load = load_item(bytes, cx);
    load.outcome.value.as_ref()?;
    load.identifier
}

/// A load rejected before the chain ran.
fn rejected_load(diagnostics: Diagnostics) -> ItemLoad {
    ItemLoad {
        outcome: LoadOutcome {
            value: None,
            original_version: known::V0_0_0,
            version: bedrock_addon_core::document::VersionInfo::new(
                None,
                false,
                known::V0_0_0,
                known::V0_0_0,
                false,
            ),
            diagnostics,
        },
        identifier: None,
    }
}

/// Reads one item file at `origin` ([`load_item`]): the stable document, a beta one, or the
/// rejection with the identifier the payload names.
pub(crate) fn read_item(
    origin: Origin,
    bytes: &[u8],
    cx: &LoadContext,
) -> DocumentRead<ItemFamily> {
    let ItemLoad {
        outcome,
        identifier,
    } = load_item(bytes, cx);
    let diagnostics = outcome.diagnostics.located(&origin);
    match outcome.value {
        Some(Loaded::Stable(item)) => {
            DocumentRead::Stable(Document::new(item, origin, outcome.version, diagnostics))
        }
        Some(Loaded::Beta(beta)) => {
            let id = beta.description.identifier.as_str().to_owned();
            DocumentRead::Beta(
                Some(id),
                OpenBeta::new(Document::new(beta, origin, outcome.version, diagnostics)),
            )
        }
        None => DocumentRead::Rejected(identifier, Rejected::new(origin, diagnostics)),
    }
}

/// Parses a modern format: the struct of `V` (1.26.30, 1.26.50, or the beta struct for a beta
/// document, whose members are the same).
pub(crate) fn parse_newest<const V: u32>(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<ModernItem<V>> {
    let version = modern_version(V);
    let parts = parse_parts(payload, version, cx, d)?;
    let menu_category_presence =
        super::MenuCategoryPresence::of(parts.description.menu_category.as_ref());
    let mut menu_category = parts.description.menu_category.unwrap_or_default();
    menu_category
        .category
        .get_or_insert(CreativeCategory::ItemCommandOnly);
    let components = into_component_map(parts.components);
    let mut legacy = parts.legacy;
    // The legacy reading holds a component it shares with the typed read in the form the
    // writer writes that component in, so that the written text gives the reading the file
    // gave (`16` and `{"value": 16}` are one value to both readers).
    for (key, written) in written_components(&components) {
        if let Some(held) = legacy.shared.get_mut(&key)
            && let Ok(written) = written
        {
            *held = written;
        }
    }
    Some(ModernItem {
        description: ItemDescription {
            identifier: ItemId::parse(parts.description.identifier),
            menu_category,
            menu_category_presence,
        },
        components,
        deprecated: parts.deprecated,
        events: parts.events,
        legacy,
    })
}
