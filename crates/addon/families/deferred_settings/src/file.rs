//! One settings file through its payload's version chain: the text checks, the public parse
//! stage and the document or its rejection.

use super::codes;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Phase, Severity, builders,
};
use bedrock_addon_core::document::{Document, Origin, Rejected, VersionInfo};
use bedrock_addon_core::json::{EnvelopeVersion, JsonSlice, Prepared};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    DocumentRead, Family, Loaded, Parsed, Selection, VersionChain, finish, parse_as_written, select,
};

/// The format version from which a payload's member itself is required (water and
/// atmosphere, 1.26.20); `None` when it is optional in every struct.
pub(crate) type PayloadRequired = Option<FormatVersion>;

/// The rejection of a file that is no text.
fn unreadable(origin: Origin) -> Rejected {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(Diagnostic::of(builders::syntax(
        "not UTF-8 JSON within the nesting limit",
    )));
    Rejected::located(origin, diagnostics)
}

/// The member that holds the version.
const FORMAT_VERSION: &str = "format_version";

/// The path of the root member `key`.
fn member(key: &str) -> JsonPath {
    JsonPath::new().child(key)
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
fn envelope<F: Family>(text: &str, required: PayloadRequired) -> Result<Envelope, Box<Diagnostic>> {
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
    let malformed = || Diagnostic::of(&codes::MALFORMED_VERSION).at(member(FORMAT_VERSION));
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
        let finding = if versions == 0 {
            Diagnostic::of(&codes::MEMBER_COUNT_WITHOUT_VERSION)
        } else {
            Diagnostic::of(&core_codes::DOCUMENT_MEMBER_COUNT).detail(DiagnosticDetail::Syntax(
                format!(
                    "{} root members, not \"format_version\" and a payload member",
                    keys.len()
                ),
            ))
        };
        return Err(Box::new(finding.at(JsonPath::new())));
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
            Diagnostic::of(&core_codes::NO_VALID_VERSION).at(JsonPath::new()),
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
            required,
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
fn foreign_payload<F: Family>(
    written: &FormatVersionInput,
    shown: Option<&str>,
    actual: &str,
    required: PayloadRequired,
) -> Diagnostic {
    let mismatch = || {
        // Only development builds of the game print the payload-key check's line; a release
        // build is silent, so the crate uses the line for that case.
        Diagnostic::of(core_codes::PAYLOAD_UNDER_OTHER_KEY.with([F::PAYLOAD_KEY, actual]))
            .at(JsonPath::new())
    };
    if written.beta_literal {
        return mismatch();
    }
    let versions = <F::History as VersionChain<F>>::VERSIONS;
    match select::<F>(written) {
        Selection::Struct(index) => {
            let required = versions
                .get(index)
                .zip(required)
                .is_some_and(|(version, from)| *version >= from);
            if required {
                Diagnostic::of(&core_codes::MISSING_REQUIRED_FIELD).at(member(F::PAYLOAD_KEY))
            } else {
                mismatch()
            }
        }
        _ => Diagnostic::of(core_codes::NO_PARSER_FOR_VERSION.with([shown.unwrap_or_default()]))
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
            // By legacy name: the variant and the definitions that were it.
            match finding.code.name() {
                "UnsupportedVersion" => {
                    finding.code = DiagnosticCode::from(&codes::BETA_UNSUPPORTED);
                }
                "BetaFeaturesNotEnabled" => {
                    finding.code = DiagnosticCode::from(&codes::BETA_NOT_ENABLED);
                }
                _ => {}
            }
        } else if finding.code.is(&core_codes::NO_PARSER_FOR_VERSION)
            && let Some(shown) = shown
        {
            // Below the first struct the pipeline says no parser is available
            // (`ENVELOPE.below_floor`), as above the last; both print the version as shown.
            finding.code = core_codes::NO_PARSER_FOR_VERSION.with([shown]).into();
        }
        diagnostics.push(finding);
    }
}

/// The parse stage of one file of payload `F`: the envelope rules, then the version chain's
/// struct selection and parse.
fn parse_stage<F: Family>(
    document: JsonSlice<'_>,
    cx: &LoadContext,
    required: PayloadRequired,
) -> Parsed<F> {
    let envelope = match envelope::<F>(document.text, required) {
        Ok(envelope) => envelope,
        Err(finding) => {
            let mut diagnostics = Diagnostics::default();
            diagnostics.push(*finding);
            return Parsed::from_value(
                None,
                VersionInfo::new(None, false, F::FLOOR, F::FLOOR, false),
                FormatVersionInput {
                    version: Some(F::FLOOR),
                    beta_literal: false,
                    prerelease: false,
                },
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
        parsed
            .diagnostics
            .push(Diagnostic::of(&core_codes::INVALID_TYPE).at(member(FORMAT_VERSION)));
        parsed.value = None;
    }
    parsed
}

/// Loads the file at `origin` of payload `F` (its `JsonDocument::read`): the envelope rules of
/// the game's document reader, then the version chain. No identifier is told for a failed
/// file: the stack's maps reserve nothing for one.
pub(crate) fn read<F: Family>(
    origin: Origin,
    bytes: &[u8],
    cx: &LoadContext,
    required: PayloadRequired,
) -> DocumentRead<F> {
    let Ok(prepared) = Prepared::new(bytes) else {
        return DocumentRead::Rejected(None, unreadable(origin));
    };
    match finish_file::<F>(parse_stage::<F>(prepared.slice(), cx, required), origin, cx) {
        Ok(document) => DocumentRead::Stable(document),
        Err(rejected) => DocumentRead::Rejected(None, rejected),
    }
}

/// The rest of the load of a parsed file.
fn finish_file<F: Family>(
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

/// The file's document for payload `F` as written (its `JsonDocument::parse_as_written`): the
/// struct the file's `format_version` selects, not upgraded.
pub(crate) fn as_written<F: Family>(
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
    required: PayloadRequired,
) -> Result<Document<F::History>, Rejected> {
    let origin = Origin::file(PackRef::app_data(), PackSide::Resource, path.clone());
    let Ok(prepared) = Prepared::new(bytes) else {
        return Err(unreadable(origin));
    };
    let parsed = parse_stage::<F>(prepared.slice(), cx, required);
    let mut diagnostics = parsed.diagnostics;
    diagnostics.locate(&origin);
    match parsed.value {
        Some(value) => Ok(Document::new(value, origin, parsed.version, diagnostics)),
        None => Err(Rejected::new(origin, diagnostics)),
    }
}

/// The body of `history::upgrade` for payload `F`: rebuilds the parse stage from the document
/// and finishes it.
pub(crate) fn upgrade<F: Family>(
    document: Document<F::History>,
    cx: &LoadContext,
) -> Result<Document<F::Latest>, Rejected> {
    let (value, origin, version, diagnostics) = document.into_parsed_parts();
    let written = FormatVersionInput {
        version: Some(version.effective),
        beta_literal: false,
        prerelease: false,
    };
    let at = JsonPath::new().child(F::PAYLOAD_KEY);
    let parsed = Parsed::<F>::from_value(Some(value), version, written, at, diagnostics);
    finish_file::<F>(parsed, origin, cx)
}

/// The identifier a value is indexed by: the one it names (`identifier`), unless that is the
/// empty string, which the game treats as no identifier (it takes no default from it and no
/// biome can name it).
pub(crate) fn named(identifier: Option<&str>) -> Option<&str> {
    identifier.filter(|id| !id.is_empty())
}

/// A loaded value of one of the five identifier payloads that names no identifier
/// (`identifier`): a `MissingIdentifier` information at the payload. The document loads, and
/// the stack does not index it. The game keys it by the empty string, which no biome can name,
/// and writes no line, so the finding is the crate's own information.
pub(crate) fn unnamed(
    identifier: Option<&str>,
    cx: &mut bedrock_addon_core::versioning::ValidateCx<'_>,
) {
    if named(identifier).is_none() {
        cx.diagnostics.push(
            Diagnostic::of(&core_codes::MISSING_IDENTIFIER)
                .severity(Severity::Info)
                .at(cx.at.clone())
                .phase(Phase::Validate),
        );
    }
}
