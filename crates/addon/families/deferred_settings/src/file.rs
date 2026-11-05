//! One settings file through its payload's version chain: the text checks, the public parse
//! stage and the document or its rejection.

use std::convert::Infallible;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, VanillaText,
};
use bedrock_addon_core::document::{Document, Origin, Rejected, VersionInfo};
use bedrock_addon_core::json::{
    EncodeError, EnvelopeVersion, JsonSlice, MAX_JSON_DEPTH, Value, check_depth, strip_bom,
};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    Family, Identity, Loaded, Parsed, Selection, VersionChain, VersionStruct, Versioned, finish,
    parse_as_written, select,
};

/// A payload of this module: no struct for the game's experimental builds, read by the struct
/// whose version the file names exactly.
pub(crate) trait Payload: Family<Beta = Infallible, Latest: VersionStruct + Clone> {
    /// The format version from which the payload member itself is required (water and
    /// atmosphere, 1.26.20); `None` when it is optional in every struct.
    const PAYLOAD_REQUIRED: Option<FormatVersion> = None;

    /// The identifier `value` is indexed by; `None` when it names none.
    fn identifier(value: &Self::Latest) -> Option<&str>;

    /// Whether the payload has an identifier at all (the five identifier maps).
    const NAMED: bool = true;
}

/// The text of a file, after the BOM and the depth check; `None` when it is not UTF-8 or too
/// deep.
fn text(bytes: &[u8]) -> Option<&str> {
    let bytes = strip_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH).ok()?;
    std::str::from_utf8(bytes).ok()
}

/// The rejection of a file that is no text.
fn unreadable(origin: Origin) -> Rejected {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(Diagnostic::error(DiagnosticCode::MalformedInput {
        message: "not UTF-8 JSON within the nesting limit".to_owned(),
        text: None,
    }));
    Rejected::located(origin, diagnostics)
}

/// The member that holds the version.
const FORMAT_VERSION: &str = "format_version";
/// The envelope's text for a `format_version` that is no version.
const MALFORMED_VERSION: &str = "Malformed format version";
/// The text of the game's payload-key check. Only development builds of the game print it;
/// a release build is silent, so the crate uses the text for that case.
const PAYLOAD_KEY_MISMATCH: &str =
    "The payload key does not match the expected one. Expected: '{}'. Actual: '{}'.";

/// The path of the root member `key`.
fn member(key: &str) -> JsonPath {
    let mut at = JsonPath::new();
    at.push(key);
    at
}

/// The names of the root members of `text` as written, a member written twice counted twice;
/// `None` when the text is no object the scan can follow (the reader reports that).
///
/// The one place a repeated key is seen before core's last-wins rule: the game counts the
/// root's members as written, which is how a root of two `format_version` members reaches
/// "The payload of the Minecraft Document is missing".
fn root_keys(text: &str) -> Option<Vec<String>> {
    let mut scan = Scan {
        bytes: text.as_bytes(),
        text,
        at: 0,
    };
    scan.skip();
    scan.expect(b'{')?;
    let mut keys = Vec::new();
    loop {
        scan.skip();
        if scan.peek()? == b'}' {
            return Some(keys);
        }
        let key = scan.string()?;
        keys.push(bedrock_addon_core::json::from_str::<String>(key).ok()?);
        scan.skip();
        scan.expect(b':')?;
        scan.value()?;
        if scan.peek()? == b',' {
            scan.at += 1;
        }
    }
}

/// A position in a JSON(C) text.
struct Scan<'a> {
    bytes: &'a [u8],
    text: &'a str,
    at: usize,
}

impl<'a> Scan<'a> {
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn expect(&mut self, byte: u8) -> Option<()> {
        (self.peek()? == byte).then(|| self.at += 1)
    }

    /// Skips white space and comments.
    fn skip(&mut self) {
        loop {
            let rest = &self.text[self.at..];
            let trimmed = rest.trim_start();
            self.at += rest.len() - trimmed.len();
            if let Some(comment) = trimmed.strip_prefix("//") {
                self.at += 2 + comment.find('\n').unwrap_or(comment.len());
            } else if let Some(comment) = trimmed.strip_prefix("/*") {
                self.at += 2 + comment.find("*/").map_or(comment.len(), |end| end + 2);
            } else {
                return;
            }
        }
    }

    /// A string token, quotes included.
    fn string(&mut self) -> Option<&'a str> {
        let start = self.at;
        self.expect(b'"')?;
        loop {
            match self.peek()? {
                b'"' => {
                    self.at += 1;
                    return self.text.get(start..self.at);
                }
                b'\\' => self.at += 2,
                _ => self.at += 1,
            }
        }
    }

    /// Skips one value, up to the `,` or the closing bracket after it. Iterative: the depth of
    /// the text is no depth of the stack.
    fn value(&mut self) -> Option<()> {
        let mut depth = 0_usize;
        loop {
            self.skip();
            match self.peek()? {
                b'"' => {
                    self.string()?;
                }
                b'{' | b'[' => {
                    depth += 1;
                    self.at += 1;
                }
                b'}' | b']' => {
                    if depth == 0 {
                        return Some(());
                    }
                    depth -= 1;
                    self.at += 1;
                }
                b',' if depth == 0 => return Some(()),
                _ => self.at += 1,
            }
        }
    }
}

/// What the envelope checks found in a file that goes on to the version chain.
struct Envelope {
    /// `format_version` is an array of three integers: the envelope takes it, the struct's
    /// string member does not.
    array_version: bool,
    /// The version as the game prints it in "no parser available for version %s": a version
    /// without a pre-release tag in full (`1.21` is `1.21.0`), any other text as written;
    /// `None` when the file has no envelope the checks could read.
    shown: Option<String>,
    /// `format_version` is the literal `"beta"`, which is no version to the game's envelope.
    beta_literal: bool,
}

/// The envelope rules of the game's document reader, in its order: an unreadable
/// `format_version`, a root of other than two members, two `format_version` members, no
/// `format_version`, and a second member that is not the payload. `Err` is the file's one
/// finding. A text that is no JSON object passes: the parse stage
/// reports it.
fn envelope<F: Payload>(text: &str) -> Result<Envelope, Box<Diagnostic>> {
    let passed = Envelope {
        array_version: false,
        shown: None,
        beta_literal: false,
    };
    let cleaned = bedrock_addon_core::json::last_wins(text);
    let Ok(probe) = bedrock_addon_core::json::probe_envelope(JsonSlice::new(&cleaned)) else {
        return Ok(passed);
    };
    let Some(keys) = root_keys(text) else {
        return Ok(passed);
    };
    let malformed = || {
        Diagnostic::error(DiagnosticCode::MalformedFormatVersion {
            text: Some(VanillaText::Fixed(MALFORMED_VERSION)),
        })
        .at(member(FORMAT_VERSION))
    };
    let (written, array_version) = match &probe.format_version {
        EnvelopeVersion::Absent => (None, false),
        EnvelopeVersion::String(text) => (
            Some(FormatVersionInput::parse(text).ok_or_else(|| Box::new(malformed()))?),
            false,
        ),
        EnvelopeVersion::Array(parts) => {
            let [Ok(major), Ok(minor), Ok(patch)] = parts.map(u16::try_from) else {
                return Err(Box::new(malformed()));
            };
            let version = FormatVersionInput {
                version: Some(FormatVersion::new(major, minor, patch)),
                beta_literal: false,
                prerelease: false,
            };
            (Some(version), true)
        }
        _ => return Err(Box::new(malformed())),
    };
    let versions = keys.iter().filter(|key| *key == FORMAT_VERSION).count();
    if keys.len() != 2 {
        let code = if versions == 0 {
            DiagnosticCode::MissingFormatVersion {
                text: Some(VanillaText::DocumentMemberCount),
            }
        } else {
            DiagnosticCode::MalformedInput {
                message: format!(
                    "{} root members, not \"format_version\" and a payload member",
                    keys.len()
                ),
                text: Some(VanillaText::DocumentMemberCount),
            }
        };
        return Err(Box::new(Diagnostic::error(code).at(JsonPath::new())));
    }
    if versions == 2 {
        return Err(Box::new(
            Diagnostic::error(bedrock_addon_core::diagnostic::cereal::payload_missing())
                .at(JsonPath::new()),
        ));
    }
    let Some(written) = written.filter(|_| versions == 1) else {
        // Two members, neither the version.
        return Err(Box::new(
            Diagnostic::error(DiagnosticCode::MissingFormatVersion {
                text: Some(VanillaText::NoValidVersion),
            })
            .at(JsonPath::new()),
        ));
    };
    // A version without a pre-release tag is printed in full; any other text as written.
    let shown = match (&probe.format_version, written.version) {
        (EnvelopeVersion::String(text), _) if written.prerelease || written.beta_literal => {
            Some(text.clone())
        }
        (_, Some(version)) => Some(version.to_string()),
        _ => None,
    };
    let actual = keys.iter().find(|key| *key != FORMAT_VERSION);
    if let Some(actual) = actual.filter(|key| *key != F::PAYLOAD_KEY) {
        return Err(Box::new(foreign_payload::<F>(
            &written,
            shown.as_deref(),
            actual,
        )));
    }
    Ok(Envelope {
        array_version,
        shown,
        beta_literal: written.beta_literal,
    })
}

/// The finding of a second root member named `actual`, not the payload key. The game only
/// asserts there (a release build is silent) and parses the root struct without its payload:
/// a version with no struct has no parser; a struct that requires the payload fails on the
/// missing member (water and atmosphere, 1.26.20); one whose payload is optional loads the
/// game's defaults under no identifier, where the crate reports a `MissingPayload` error with
/// the assert's text.
fn foreign_payload<F: Payload>(
    written: &FormatVersionInput,
    shown: Option<&str>,
    actual: &str,
) -> Diagnostic {
    let mismatch = || {
        Diagnostic::error(DiagnosticCode::MissingPayload {
            text: Some(VanillaText::Formatted {
                template: PAYLOAD_KEY_MISMATCH,
                args: vec![F::PAYLOAD_KEY.to_owned(), actual.to_owned()],
            }),
        })
        .at(JsonPath::new())
    };
    if written.beta_literal {
        return mismatch();
    }
    let versions = <F::Any as VersionChain<F>>::VERSIONS;
    match select::<F>(written) {
        Selection::Struct(index) => {
            let required = versions
                .get(index)
                .zip(F::PAYLOAD_REQUIRED)
                .is_some_and(|(version, from)| *version >= from);
            if required {
                Diagnostic::error(DiagnosticCode::MissingRequiredField { text: None })
                    .at(member(F::PAYLOAD_KEY))
            } else {
                mismatch()
            }
        }
        _ => Diagnostic::error(DiagnosticCode::UnsupportedVersion {
            text: Some(VanillaText::NoParserForVersion {
                version: shown.unwrap_or_default().to_owned(),
            }),
        })
        .at(member(FORMAT_VERSION)),
    }
}

/// Gives the version findings of the parse stage the game's text: below the first struct
/// there is no parser either (no "use json format version … or higher": every parser's range
/// is its own version), and the version is printed in full.
///
/// The literal `"beta"` is no version to the game: its envelope refuses the file as
/// "Malformed format version". The module has no struct for the game's experimental builds,
/// so the chain says `BetaFeaturesNotEnabled` without the toggle and `UnsupportedVersion`
/// with it; both carry the game's text (the crate's own finding code, vanilla's text).
fn version_texts(diagnostics: &mut Diagnostics, envelope: &Envelope) {
    let shown = envelope.shown.as_deref();
    let found = std::mem::take(diagnostics);
    for mut finding in found {
        if envelope.beta_literal {
            if let DiagnosticCode::UnsupportedVersion { text }
            | DiagnosticCode::BetaFeaturesNotEnabled { text } = &mut finding.code
            {
                *text = Some(VanillaText::Fixed(MALFORMED_VERSION));
            }
        } else if let DiagnosticCode::UnsupportedVersion { text: Some(text) } = &mut finding.code {
            match text {
                VanillaText::UseFormatVersion { .. } => {
                    if let Some(version) = shown {
                        *text = VanillaText::NoParserForVersion {
                            version: version.to_owned(),
                        };
                    }
                }
                VanillaText::NoParserForVersion { version } => {
                    if let Some(shown) = shown {
                        shown.clone_into(version);
                    }
                }
                _ => {}
            }
        }
        diagnostics.push(finding);
    }
}

/// The parse stage of one file of payload `F`: the envelope rules, then the version chain's
/// struct selection and parse.
fn parse_stage<F: Payload>(document: JsonSlice<'_>, cx: &LoadContext) -> Parsed<F> {
    let envelope = match envelope::<F>(document.text) {
        Ok(envelope) => envelope,
        Err(finding) => {
            let mut diagnostics = Diagnostics::default();
            diagnostics.push(*finding);
            return Parsed::new(
                None,
                VersionInfo::new(None, false, F::FLOOR, F::FLOOR, false),
                FormatVersionInput {
                    version: Some(F::FLOOR),
                    beta_literal: false,
                    prerelease: false,
                },
                Identity::default(),
                ActorType::Mob,
                false,
                JsonPath::new(),
                diagnostics,
            );
        }
    };
    let mut parsed = parse_as_written::<F>(document, cx);
    version_texts(&mut parsed.diagnostics, &envelope);
    if envelope.array_version && parsed.value.is_some() {
        // The envelope reads `[1, 21, 40]`; the struct's own `format_version` member is a
        // string.
        parsed.diagnostics.push(
            Diagnostic::error(DiagnosticCode::InvalidType { text: None })
                .at(member(FORMAT_VERSION)),
        );
        parsed.value = None;
    }
    parsed
}

/// Loads one file of payload `F`.
pub(crate) fn parse_file<F: Payload>(
    pack: PackRef,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<F::Latest>, Rejected> {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let origin = Origin::file(pack, PackSide::Resource, path.clone());
    let Some(text) = text(bytes) else {
        return Err(unreadable(origin));
    };
    finish_file::<F>(parse_stage::<F>(JsonSlice::new(text), cx), origin, cx)
}

/// The rest of the load of a parsed file.
fn finish_file<F: Payload>(
    parsed: Parsed<F>,
    origin: Origin,
    cx: &LoadContext,
) -> Result<Document<F::Latest>, Rejected> {
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    diagnostics.locate(&origin);
    match outcome.value {
        Some(Loaded::Stable(value)) => {
            Ok(Document::new(value, origin, outcome.version, diagnostics))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}

/// The file's document for payload `F` as written: the struct the file's `format_version`
/// selects, not upgraded.
pub(crate) fn as_written<F: Payload>(
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<F::Any>, Rejected> {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let origin = Origin::file(PackRef::app_data(), PackSide::Resource, path.clone());
    let Some(text) = text(bytes) else {
        return Err(unreadable(origin));
    };
    let parsed = parse_stage::<F>(JsonSlice::new(text), cx);
    let mut diagnostics = parsed.diagnostics;
    diagnostics.locate(&origin);
    match parsed.value {
        Some(value) => Ok(Document::new(value, origin, parsed.version, diagnostics)),
        None => Err(Rejected::new(origin, diagnostics)),
    }
}

/// The body of `history::upgrade` for payload `F`: rebuilds the parse stage from the document
/// and finishes it.
pub(crate) fn upgrade<F: Payload>(
    document: Document<F::Any>,
    cx: &LoadContext,
) -> Result<Document<F::Latest>, Rejected> {
    let origin = document.origin().clone();
    let version = document.version().clone();
    let diagnostics = document.diagnostics().clone();
    let value = document.into_value();
    let written = FormatVersionInput {
        version: Some(version.effective),
        beta_literal: false,
        prerelease: false,
    };
    let mut at = JsonPath::new();
    at.push(F::PAYLOAD_KEY);
    let parsed = Parsed::<F>::new(
        Some(value),
        version,
        written,
        Identity::default(),
        ActorType::Mob,
        false,
        at,
        diagnostics,
    );
    finish_file::<F>(parsed, origin, cx)
}

/// The whole document of `value` at the newest format version of payload `F`.
pub(crate) fn document<F: Payload>(value: &F::Latest) -> Result<Value, EncodeError> {
    let mut root = facet_value::VObject::new();
    root.insert(
        "format_version",
        <F::Latest as Versioned>::VERSION.to_string(),
    );
    root.insert(F::PAYLOAD_KEY, value.write()?);
    Ok(root.into())
}

/// The identifier `value` is indexed by: the one it names, unless that is the empty string,
/// which the game treats as no identifier (it takes no default from it and no biome can name
/// it).
pub(crate) fn named<F: Payload>(value: &F::Latest) -> Option<&str> {
    F::identifier(value).filter(|id| !id.is_empty())
}

/// A loaded value of one of the five identifier payloads that names no identifier: a
/// `MissingIdentifier` information at the payload. The document loads, and the stack does not
/// index it. The game keys it by the empty string, which no biome can name, and writes no
/// line, so the finding is the crate's own information.
pub(crate) fn unnamed<F: Payload>(
    value: &F::Latest,
    cx: &mut bedrock_addon_core::versioning::ValidateCx<'_>,
) {
    if F::NAMED && named::<F>(value).is_none() {
        cx.diagnostics.push(
            Diagnostic::info(DiagnosticCode::MissingIdentifier { text: None })
                .at(cx.at.clone())
                .phase(Phase::Validate),
        );
    }
}
