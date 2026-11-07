//! One file through the chain: envelope → version → parse
//! ([`parse_as_written`]) → upgrade → validate ([`finish`]).

use std::fmt;

use facet::Facet;
use facet_json::RawJson;
use indexmap::IndexMap;

use super::component::validate_components;
use super::payload::span_of;
use super::range::{Selection, select};
use super::{
    BETA_VERSION, BelowFloor, BetaPermission, EnvelopeCheck, Family, Identity, LoadOutcome, Loaded,
    MissingVersion, ParseCx, Reading, StructIndex, UpgradeCx, ValidateCx, VersionChain,
    VersionInput,
};
use crate::diagnostic::builders::syntax;
use crate::diagnostic::codes as core_codes;
use crate::diagnostic::{
    Code, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, Location, Phase, Severity, Span,
};
use crate::document::VersionInfo;
use crate::json::{EnvelopeVersion, JsonSlice, RootProbe, Value, probe_envelope};
use crate::load::LoadContext;
use crate::molang::{MolangContext, MolangFields, not_checked};
use crate::pack_identity::PackPath;
use crate::vanilla::ActorType;
use crate::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput, PickedVersion};

/// The member that holds the version.
const FORMAT_VERSION: &str = "format_version";

/// Loads one modern document of family `F`: [`parse_as_written`], then [`finish`].
/// `document` is the file text after `blank_bom` / `check_depth` (an MCB document as
/// written-out text). The value is `None` when the document is rejected; then `diagnostics`
/// holds why (an error, or an `EmptyDocument` at the family's severity).
pub fn load<F: Family>(document: JsonSlice<'_>, cx: &LoadContext) -> LoadOutcome<F> {
    finish(parse_as_written::<F>(document, cx), cx)
}

/// [`load`] of one file at `path`, closed: `Family::close` runs once at the end, told whether
/// the file failed (no value). The unclosed stages ([`load`], [`parse_as_written`],
/// [`finish`]) never close.
pub fn load_file<F: Family>(
    document: JsonSlice<'_>,
    path: &PackPath,
    cx: &LoadContext,
) -> LoadOutcome<F> {
    finish_file(
        parse_as_written::<F>(document, cx),
        Some(document),
        path,
        cx,
    )
}

/// [`parse_as_written`] of one file at `path`, the first of two stages: a parse that gives no
/// value ends the file, and is closed here (`failed`); a parse that gives one is not closed,
/// since [`finish_file`] closes the file once at its end. Either way a file is closed once.
///
/// A parse with no value is already closed and must not be followed by [`finish_file`],
/// which would close it a second time. A caller that ends the file at a successful parse,
/// with no [`finish_file`] after it, closes the file itself.
pub fn parse_file_as_written<F: Family>(
    document: JsonSlice<'_>,
    path: &PackPath,
    cx: &LoadContext,
) -> Parsed<F> {
    let mut parsed = parse_as_written::<F>(document, cx);
    if parsed.value.is_none() {
        F::close(&mut parsed.diagnostics, Some(document), path, true);
    }
    parsed
}

/// [`finish`] of one file's parse at `path`, closed with whether the file failed. `document`
/// is the text, when the parse was of one (`None` for a document rebuilt from a value).
pub fn finish_file<F: Family>(
    parsed: Parsed<F>,
    document: Option<JsonSlice<'_>>,
    path: &PackPath,
    cx: &LoadContext,
) -> LoadOutcome<F> {
    let mut outcome = finish(parsed, cx);
    let failed = outcome.value.is_none();
    F::close(&mut outcome.diagnostics, document, path, failed);
    outcome
}

/// A document parsed by the struct its `format_version` selects, not upgraded: what
/// [`parse_as_written`] gives and [`finish`] takes. The typed `parse_as_written` and
/// `history::<Name>History::upgrade` of a family's document are built on the pair. Another crate builds
/// one with [`from_value`](Self::from_value).
pub struct Parsed<F: Family> {
    /// The chain value, or `None` when the document was rejected (then `diagnostics` says why).
    pub value: Option<F::History>,
    /// What the file said about its version and which struct parsed it (`upgraded` is set by
    /// [`finish`]).
    pub version: VersionInfo,
    /// The version the Molang fields whose version is the document's take: the envelope's (or
    /// the payload's, `Family::payload_version`), the effective version when absent.
    pub written: FormatVersionInput,
    /// `Family::identity` of the payload.
    pub identity: Identity,
    /// `Family::actor_type` of the identity.
    pub actor_type: ActorType,
    /// The payload has `Family::SKIP_UPGRADES_KEY`.
    pub skip_upgrades: bool,
    /// The payload's path: the payload key, or the root on the raw-text path.
    pub at: JsonPath,
    /// Every finding so far.
    pub diagnostics: Diagnostics,
    /// The warnings of the clamp of the numbers no double holds, as they are among
    /// `diagnostics`: [`finish`] takes one back where a later phase gives a finding at the
    /// same number.
    pub(crate) clamp: Diagnostics,
}

impl<F: Family> Parsed<F> {
    /// The parse of a document a family read by hand: its chain value, what the file said
    /// about its version, the version its Molang fields take, the payload's path and the
    /// findings so far. The identity is empty, the actor type `Mob` and the skip key absent
    /// until [`with_identity`](Self::with_identity) and [`with_actor`](Self::with_actor) say
    /// otherwise.
    pub fn from_value(
        value: Option<F::History>,
        version: VersionInfo,
        written: FormatVersionInput,
        at: JsonPath,
        diagnostics: Diagnostics,
    ) -> Self {
        Self {
            value,
            version,
            written,
            identity: Identity::default(),
            actor_type: ActorType::Mob,
            skip_upgrades: false,
            at,
            diagnostics,
            clamp: Diagnostics::default(),
        }
    }

    /// The parse with the payload's identity (`Family::identity`).
    #[must_use]
    pub fn with_identity(self, identity: Identity) -> Self {
        Self { identity, ..self }
    }

    /// The parse with the actor type the steps see and whether the payload has the skip key
    /// (`Family::SKIP_UPGRADES_KEY`).
    #[must_use]
    pub fn with_actor(self, actor_type: ActorType, skip_upgrades: bool) -> Self {
        Self {
            actor_type,
            skip_upgrades,
            ..self
        }
    }

    /// `UpgradeCx::skip_section_a` of the remaining steps, which suppresses the version-gated
    /// upgrade ops: set by the skip key, or for a document at or above the engine version.
    pub fn skip_section_a(&self) -> bool {
        self.skip_upgrades || self.version.effective >= ENGINE_VERSION
    }
}

impl<F: Family> fmt::Debug for Parsed<F>
where
    F::History: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Parsed")
            .field("value", &self.value)
            .field("version", &self.version)
            .field("written", &self.written)
            .field("identity", &self.identity)
            .field("actor_type", &self.actor_type)
            .field("skip_upgrades", &self.skip_upgrades)
            .field("at", &self.at)
            .field("diagnostics", &self.diagnostics)
            .field("clamp", &self.clamp)
            .finish()
    }
}

/// Parses one modern document of family `F` without upgrading it: the envelope checks, the
/// version (envelope, else `Family::payload_version`, else `ENVELOPE.missing`), the beta
/// permission, struct selection by the range model and `Family::route`, the parse with Molang
/// pass 1, and the identity, actor type and skip key the steps need.
pub fn parse_as_written<F: Family>(document: JsonSlice<'_>, cx: &LoadContext) -> Parsed<F> {
    let mut told = Diagnostics::default();
    // The game's document input sizes the text first. An MCB source is a binary input, so it
    // gets no JSON size check.
    let json_document = F::ENVELOPE.sizes_json(cx.source);
    let too_large = json_document
        && (size_finding(document.text.len(), cx.max_json_document_bytes, &mut told)
            || rapidjson_refuses::<F>(document, &mut told));
    if too_large {
        let identity = Identity::default();
        return Parsed {
            value: None,
            version: VersionInfo::new(None, false, F::FLOOR, F::FLOOR, false),
            written: FormatVersionInput {
                version: Some(F::FLOOR),
                beta_literal: false,
                prerelease: false,
            },
            actor_type: F::actor_type(&identity),
            identity,
            skip_upgrades: false,
            at: JsonPath::new(),
            diagnostics: told,
            clamp: Diagnostics::default(),
        };
    }
    // Of a member written twice the last value stands, whole, at every level; the earlier
    // copies are blanked so offsets stay.
    let cleaned = crate::json::last_wins_except(document.text, F::ENVELOPE.duplicates_kept_in);
    let cleaned = JsonSlice {
        text: &cleaned,
        offset: document.offset,
    };
    // A number no double holds is read as the largest finite one where the family keeps
    // what it does not read: every reader below is given the clamped text.
    let mut beyond = Diagnostics::default();
    // Every copy of a member written twice is clamped; the warning is for the copy the
    // family keeps.
    let mut clamped = if F::ENVELOPE.clamps_beyond_double {
        crate::json::clamp_tokens(
            cleaned,
            &crate::json::ClampPlan::doubles().first_kept_in(F::ENVELOPE.duplicates_kept_in),
            &JsonPath::new(),
            &mut beyond,
        )
    } else {
        None
    };
    // The version of a document-input family is picked from the text as written, before a
    // duplicated member is blanked.
    let as_written = document.text;
    let (mut run, mut value) = read::<F>(cleaned, clamped.as_ref(), as_written, cx);
    // A float member that read a number beyond its width holds an infinity and says where
    // (`json::Misread`): the number is clamped in the text at the width of a float, and the
    // document is read again, once, so that the member holds what the warning says.
    if value.is_some()
        && let Some((text, told)) = clamp_floats::<F>(cleaned, &run.diagnostics)
    {
        (run, value) = read::<F>(cleaned, Some(&text), as_written, cx);
        clamped = Some(text);
        beyond = told;
    }
    let mut clamp = Diagnostics::default();
    if let Some(clamped) = &clamped {
        // The findings so far name places of the clamped text: back to the file as written.
        // A member that says the number itself (a float holds the largest `f32`, an integer
        // does not take it) has said it: once is enough.
        clamp = clamped.settle(&mut run.diagnostics, cleaned.offset, beyond);
    }
    let actor_type = F::actor_type(&run.identity);
    // With the moves a reader recorded: a later phase names a member it moved as the file
    // has it.
    told.append(run.diagnostics);
    Parsed {
        value,
        version: run.version,
        written: run.written,
        identity: run.identity,
        actor_type,
        skip_upgrades: run.skip_upgrades,
        at: run.at,
        diagnostics: told,
        clamp,
    }
}

/// One reading of the document `cleaned`, in the text `clamped` when numbers were clamped: the
/// run with its findings (which name places of the text read) and the value, `None` with an
/// error.
fn read<F: Family>(
    cleaned: JsonSlice<'_>,
    clamped: Option<&crate::json::ClampedText>,
    as_written: &str,
    cx: &LoadContext,
) -> (Run, Option<F::History>) {
    let mut run = Run {
        diagnostics: Diagnostics::default(),
        version: VersionInfo::new(None, false, F::FLOOR, F::FLOOR, false),
        written: FormatVersionInput {
            version: Some(F::FLOOR),
            beta_literal: false,
            prerelease: false,
        },
        identity: Identity::default(),
        skip_upgrades: false,
        at: JsonPath::new(),
    };
    let document = JsonSlice {
        text: clamped.map_or(cleaned.text, |clamped| &clamped.text),
        offset: cleaned.offset,
    };
    let value = run
        .parse::<F>(document, as_written, cx)
        .filter(|_| !run.diagnostics.has_errors());
    // Content after the root: the game's JSON reader never looks past the root value, so the
    // document loads; the author is told what is not read, an information. (A document-input
    // family that parses with RapidJSON refuses such a text earlier, in `rapidjson_refuses`.)
    run.diagnostics
        .extend(crate::json::trailing_content_finding(document));
    (run, value)
}

/// The text of `cleaned` with the numbers clamped that a first reading found beyond the float
/// member that holds them (the warnings of `json::Misread::finding` among `found`: at such a
/// path the number is read at the width of a float), and, for a family that clamps them, the
/// numbers no double holds; with the warnings of every clamped number, each on its token.
/// `None` when the first reading told of no such member.
fn clamp_floats<F: Family>(
    cleaned: JsonSlice<'_>,
    found: &Diagnostics,
) -> Option<(crate::json::ClampedText, Diagnostics)> {
    use crate::json::Width;
    let floats: std::collections::HashSet<JsonPath> = found
        .iter()
        .filter(|finding| finding.is(&crate::diagnostic::codes::BEYOND_FLOAT_MEMBER))
        .map(|finding| finding.location.json_path.clone())
        .collect();
    if floats.is_empty() {
        return None;
    }
    let mut told = Diagnostics::default();
    let width = |path: &[crate::diagnostic::JsonPathSegment<'_>]| {
        let mut at = JsonPath::new();
        for segment in path {
            match segment {
                crate::diagnostic::JsonPathSegment::Key(key) => at.push(key),
                crate::diagnostic::JsonPathSegment::Index(index) => at.push_index(*index),
            }
        }
        if floats.contains(&at) {
            Some(Width::Float)
        } else {
            F::ENVELOPE.clamps_beyond_double.then_some(Width::Double)
        }
    };
    let text = crate::json::clamp_tokens(
        cleaned,
        &crate::json::ClampPlan::widths(&width).first_kept_in(F::ENVELOPE.duplicates_kept_in),
        &JsonPath::new(),
        &mut told,
    )?;
    let clamped_a_float = told
        .iter()
        .any(|finding| floats.contains(&finding.location.json_path));
    clamped_a_float.then_some((text, told))
}

/// The rest of [`load`] for a parsed document: the upgrade walk, the generic checks on the
/// newest struct (each section `F::component_sections` lists; a failed optional section is
/// listed in `ValidateCx::skipped`), Molang pass 2 and `F::validate`. A rejected parse passes
/// through with its findings. The value is `None` when anything raised an error.
pub fn finish<F: Family>(parsed: Parsed<F>, cx: &LoadContext) -> LoadOutcome<F> {
    let skip_section_a = parsed.skip_section_a();
    let Parsed {
        value,
        mut version,
        written,
        identity,
        actor_type,
        at,
        mut diagnostics,
        clamp,
        ..
    } = parsed;
    let original_version = version.effective;
    let reading = parsed_reading::<F>(&version, cx.source);
    let value = value.and_then(|any| {
        let versions = <F::History as VersionChain<F>>::VERSIONS;
        version.upgraded = !version.beta && versions.last() != Some(&version.parsed_as);
        let identifier = identity.identifier.clone().unwrap_or_default();
        let mut upgrade = UpgradeCx {
            load: cx,
            original_version,
            identifier: &identifier,
            actor_type,
            skip_section_a,
            reading,
            diagnostics: &mut diagnostics,
        };
        let mut loaded = any.upgrade_to_end(&mut upgrade);
        if diagnostics.has_errors() {
            return None;
        }
        validate::<F>(
            &mut loaded,
            &written,
            original_version,
            &at,
            reading,
            cx,
            &mut diagnostics,
        );
        (!diagnostics.has_errors()).then_some(loaded)
    });
    // What a step moved is named as the file has it, before any finding is taken
    // out of the list.
    diagnostics.name_as_the_file_has_it();
    // A step or a check that speaks of a number the clamp has warned of has the word: the
    // member does not hold "the largest finite double" there.
    crate::json::drop_spoken_for(&mut diagnostics, &clamp);
    LoadOutcome {
        value,
        original_version,
        version,
        diagnostics,
    }
}

/// The reading of the slice that parsed a document (`version.parsed_as`, the beta struct at
/// [`BETA_VERSION`]), which the steps and the checks grade by; [`Reading::OWN`] for a version
/// that is no slice of the chain (a family's own parse that names none).
fn parsed_reading<F: Family>(
    version: &VersionInfo,
    source: crate::load::DocumentSource,
) -> Reading {
    let index = if version.parsed_as == BETA_VERSION {
        StructIndex::Beta
    } else {
        let versions = <F::History as VersionChain<F>>::VERSIONS;
        match versions.iter().position(|v| *v == version.parsed_as) {
            Some(i) => StructIndex::Stable(i),
            None => return Reading::OWN,
        }
    };
    super::reading::slice_reading::<F>(index, source)
}

/// The generic checks, Molang pass 2 and `F::validate` on the loaded value.
fn validate<F: Family>(
    loaded: &mut Loaded<F>,
    written: &FormatVersionInput,
    original_version: FormatVersion,
    at: &JsonPath,
    reading: Reading,
    cx: &LoadContext,
    d: &mut Diagnostics,
) {
    let mut skipped = Vec::new();
    if let Loaded::Stable(latest) = loaded {
        for section in F::component_sections(latest) {
            let path = section.at.clone();
            let optional = section.optional;
            let passed = validate_components::<F>(section, original_version, cx, at, d);
            if !passed && optional {
                skipped.push(path);
            }
        }
    }
    #[cfg(feature = "molang")]
    let molang_failures = molang_pass_two::<F>(loaded, written, at, cx, d);
    #[cfg(not(feature = "molang"))]
    molang_pass_two::<F>(loaded, written, at, cx, d);
    let mut validate = ValidateCx {
        load: cx,
        original_version,
        at: at.clone(),
        skipped,
        #[cfg(feature = "molang")]
        molang_failures,
        reading,
        diagnostics: d,
    };
    F::validate(loaded, &mut validate);
}

/// Molang pass 2 on the loaded value, through the chain's walk: with
/// `LoadContext::molang`, the compile check on the side of the pack (a resource pack compiles
/// for the client); otherwise one `MolangNotChecked` info when the value holds expressions.
/// Returns the fields that did not pass, for `F::validate`'s failure rules.
#[cfg(feature = "molang")]
fn molang_pass_two<F: Family>(
    loaded: &mut Loaded<F>,
    written: &FormatVersionInput,
    at: &JsonPath,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Vec<crate::molang::FieldOutcome> {
    use crate::molang::{Side, check_fields};
    use crate::pack_identity::PackType;
    let base = Location::new(None, None, at.clone(), None);
    let molang = molang_context(written, cx);
    let side = if cx.pack_type == PackType::Resource {
        Side::Client
    } else {
        Side::Server
    };
    let mut failures = Vec::new();
    <F::History as VersionChain<F>>::with_molang(loaded, &mut |fields: &mut dyn MolangFields| {
        match &cx.molang {
            Some(check) => failures = check_fields(fields, check, side, &molang, &base, d),
            None => {
                not_checked(fields, &base, d);
            }
        }
    });
    failures
}

/// Molang pass 2 without the `molang` feature: one `MolangNotChecked` info when the loaded
/// value holds expressions.
#[cfg(not(feature = "molang"))]
fn molang_pass_two<F: Family>(
    loaded: &mut Loaded<F>,
    written: &FormatVersionInput,
    at: &JsonPath,
    cx: &LoadContext,
    d: &mut Diagnostics,
) {
    let _ = (written, cx);
    let base = Location::new(None, None, at.clone(), None);
    <F::History as VersionChain<F>>::with_molang(loaded, &mut |fields: &mut dyn MolangFields| {
        not_checked(fields, &base, d);
    });
}

/// The Molang context of a document: the pack's version from the load context, the
/// document's from what the envelope said.
pub(super) fn molang_context(written: &FormatVersionInput, cx: &LoadContext) -> MolangContext {
    MolangContext::new(
        cx.min_engine_version.as_ref(),
        cx.molang_experiments,
        written,
        Some(cx.molang_version),
    )
}

/// The version an envelope gives, without loading it: `None` when absent, unreadable, `"beta"`
/// or the text is not an object. What a caller routing between two families of one payload
/// key routes by.
pub fn file_version(document: JsonSlice<'_>) -> Option<FormatVersion> {
    let envelope = probe_envelope(document).ok()?;
    read_version(&envelope.format_version).ok()??.version
}

/// The version a member `key` of the payload object holds (a string or three integers, as
/// the envelope's), for `Family::payload_version` (entity: `minecraft:entity.format_version`).
/// `None` when absent or unreadable.
pub fn member_version(payload: JsonSlice<'_>, key: &str) -> Option<FormatVersionInput> {
    let members =
        crate::json::from_str_borrowed_jsonc::<IndexMap<String, RawJson<'_>>>(payload.text).ok()?;
    let json = crate::json::from_str_jsonc::<Value>(members.get(key)?.as_str()).ok()?;
    FormatVersionInput::from_json(&json)
}

/// The version as the game's "no parser available for version …" text prints it, the
/// semantic-version string of what the input parsed: `1.26` and `1` are `1.26.0` and `1.0.0`,
/// build metadata is dropped; the beta literal and a pre-release are given as `written`. The
/// one rendering of that text for the framework and the families that remap a
/// finding to it (DDUI, particle).
#[must_use]
pub fn no_parser_version(input: &FormatVersionInput, written: &str) -> String {
    match input.version {
        Some(version) if !input.prerelease && !input.beta_literal => version.to_string(),
        _ => written.to_owned(),
    }
}

/// The finding of a version no struct covers, with the game's "no parser" text.
pub(super) fn unsupported(written: &str, at: JsonPath) -> Diagnostic {
    error(core_codes::NO_PARSER_FOR_VERSION.with([written]), at, None)
}

/// The default identity: `description.identifier` and `description.runtime_identifier`.
pub(super) fn description_identity(payload: JsonSlice<'_>) -> Identity {
    /// The members the identity is read from; everything else is ignored.
    #[derive(Facet)]
    struct Shell {
        #[facet(default)]
        description: Option<Value>,
    }
    let description = crate::json::from_str_jsonc::<Shell>(payload.text)
        .ok()
        .and_then(|shell| shell.description);
    let member = |key: &str| {
        let value = description.as_ref()?.as_object()?.get(key)?;
        Some(value.as_string()?.as_str().to_owned())
    };
    Identity {
        identifier: member("identifier"),
        runtime_identifier: member("runtime_identifier"),
    }
}

/// The state of one parse.
struct Run {
    diagnostics: Diagnostics,
    version: VersionInfo,
    written: FormatVersionInput,
    identity: Identity,
    skip_upgrades: bool,
    at: JsonPath,
}

impl Run {
    fn parse<F: Family>(
        &mut self,
        document: JsonSlice<'_>,
        as_written: &str,
        cx: &LoadContext,
    ) -> Option<F::History> {
        let envelope = self.envelope::<F>(document)?;
        let (read, as_written) = match F::ENVELOPE.input.version_input() {
            VersionInput::Root => (
                read_version(&envelope.format_version),
                envelope.format_version.clone(),
            ),
            VersionInput::Document => picked_version(as_written),
        };
        let Ok(written) = read else {
            self.diagnostics.push(error(
                Code::from(&core_codes::MALFORMED_FORMAT_VERSION),
                version_at(),
                None,
            ));
            return None;
        };
        let (payload, at) = self.payload::<F>(document, &envelope, written.as_ref())?;
        self.at = at;
        // The payload object proper, also on the raw path (the identity, the skip key and a
        // payload version are members of it).
        let object = envelope.payload(F::PAYLOAD_KEY).unwrap_or(payload);
        self.identity = F::identity(object);
        self.skip_upgrades = F::SKIP_UPGRADES_KEY.is_some_and(|key| has_member(object, key));
        let (written, index) = self.struct_index::<F>(written, &as_written, payload, object, cx)?;
        self.written = written;

        let versions = <F::History as VersionChain<F>>::VERSIONS;
        self.version.parsed_as = match index {
            StructIndex::Stable(i) => versions.get(i).copied().unwrap_or(F::FLOOR),
            StructIndex::Beta => BETA_VERSION,
        };
        // The selected slice's reading: its null rule, its scalar rules and the reading itself
        // (which grades an unknown member) ride on the context into `parse_payload` and
        // `parse_component` (a rule the caller's context carries does not leak in; an MCB
        // source is read by the binary reader, so by default it has no null rule), and so does
        // whether a component that fails to read is fatal.
        let reading = super::reading::slice_reading::<F>(index, cx.source);
        let rule = reading.null_rule();
        let strict_components = reading.components_fatal;
        let scalar_rules = reading.scalar_rules();
        let keeps_unknown = reading.keeps_unknown;
        let scoped;
        let load = if cx.null_rule == rule
            && cx.strict_components == strict_components
            && cx.scalar_rules == scalar_rules
            && cx.keeps_unknown == keeps_unknown
            && cx.reading == reading
        {
            cx
        } else {
            let mut with_rule = cx.clone();
            with_rule.null_rule = rule;
            with_rule.strict_components = strict_components;
            with_rule.scalar_rules = scalar_rules;
            with_rule.keeps_unknown = keeps_unknown;
            with_rule.reading = reading;
            scoped = with_rule;
            &scoped
        };
        let parse = ParseCx {
            load,
            written,
            original_version: self.version.effective,
            at: self.at.clone(),
        };
        F::History::parse_struct(payload, index, &parse, &mut self.diagnostics)
    }

    /// The text the struct parses and its path: the payload under the family's key in a
    /// checked envelope (exactly two root members), or the whole document on the raw-text path.
    fn payload<'a, F: Family>(
        &mut self,
        document: JsonSlice<'a>,
        envelope: &RootProbe<'a>,
        written: Option<&FormatVersionInput>,
    ) -> Option<(JsonSlice<'a>, JsonPath)> {
        let raw = match F::ENVELOPE.check {
            EnvelopeCheck::Always | EnvelopeCheck::Uncounted => false,
            EnvelopeCheck::Never => true,
            // `minimum-beta.n` sorts below `minimum`, as it does in `select`.
            EnvelopeCheck::From(minimum) => written.is_none_or(|w| {
                !w.beta_literal
                    && w.version
                        .is_none_or(|v| v < minimum || (w.prerelease && v == minimum))
            }),
        };
        if raw {
            return Some((document, JsonPath::new()));
        }
        // Vanilla checks the shape only for a document whose version is not below the
        // loader's minimum; an absent version is below every minimum, so such a
        // document is not counted and its payload is read from under the key.
        let counted = F::ENVELOPE.check != EnvelopeCheck::Uncounted;
        if counted && written.is_some() && envelope.member_count() != 2 {
            self.diagnostics.push(
                error(
                    Code::from(&core_codes::DOCUMENT_MEMBER_COUNT),
                    JsonPath::new(),
                    Some(span_of(document)),
                )
                .detail(DiagnosticDetail::Syntax(format!(
                    "{} root members, not \"format_version\" and a payload member",
                    envelope.member_count()
                ))),
            );
            return None;
        }
        let Some(payload) = envelope.payload(F::PAYLOAD_KEY) else {
            // In a counted envelope the other member is the payload under a key the family
            // does not use, which the game refuses by name.
            let other = envelope.members.keys().next();
            let code = match other {
                Some(actual) if counted && written.is_some() => {
                    crate::diagnostic::cereal::payload_key_mismatch(F::PAYLOAD_KEY, actual)
                }
                _ => Code::from(&core_codes::MISSING_PAYLOAD),
            };
            self.diagnostics
                .push(error(code, JsonPath::new(), Some(span_of(document))));
            return None;
        };
        if written.is_none() || !counted {
            // The root of a document without a version is not counted, so it may hold more
            // than the payload: such a member is read by nothing and held nowhere, and is
            // told where it is; so is a root the family never counts. The game says nothing
            // of it, so the crate reports it as an information.
            for key in envelope.members.keys().filter(|key| *key != F::PAYLOAD_KEY) {
                let mut member = JsonPath::new();
                member.push(key);
                self.diagnostics.push(Diagnostic::new(
                    Code::from(&core_codes::UNRECOGNIZED_FIELD),
                    Severity::Info,
                    Phase::Parse,
                    Location::new(None, None, member, None),
                ));
            }
        }
        let mut at = JsonPath::new();
        at.push(F::PAYLOAD_KEY);
        Some((payload, at))
    }

    /// The struct that parses the document, and the version its Molang fields take: an absent
    /// version from `F::payload_version`, else per `F::ENVELOPE.missing`; the beta literal per
    /// the beta permission; a stable version per the range model. Records the version
    /// information on the way.
    fn struct_index<F: Family>(
        &mut self,
        written: Option<FormatVersionInput>,
        as_written: &EnvelopeVersion,
        payload: JsonSlice<'_>,
        object: JsonSlice<'_>,
        cx: &LoadContext,
    ) -> Option<(FormatVersionInput, StructIndex)> {
        // Behavior-pack entities fall back to `minecraft:entity.format_version`.
        let (written, as_written) = match written {
            None => match F::payload_version(object) {
                Some(inner) => {
                    let text = inner.version.map(|v| v.to_string()).unwrap_or_default();
                    (Some(inner), EnvelopeVersion::String(text))
                }
                None => (None, EnvelopeVersion::Absent),
            },
            some => (some, as_written.clone()),
        };
        let identity = self.identity.clone();
        match written {
            None => {
                match F::ENVELOPE.missing {
                    MissingVersion::Reject { code } => {
                        self.diagnostics
                            .push(error(Code::from(code), JsonPath::new(), None));
                        return None;
                    }
                    MissingVersion::Floor { code, severity } => {
                        if let Some(severity) = severity {
                            self.diagnostics.push(Diagnostic::new(
                                Code::from(code),
                                severity,
                                Phase::Parse,
                                Location::new(None, None, JsonPath::new(), None),
                            ));
                        }
                    }
                }
                let effective = F::FLOOR;
                self.version.effective = effective;
                let written = FormatVersionInput {
                    version: Some(effective),
                    beta_literal: false,
                    prerelease: false,
                };
                let text = effective.to_string();
                let index = self.stable::<F>(&written, &text, payload, &identity, cx)?;
                Some((written, index))
            }
            Some(written) if written.beta_literal => {
                self.version.beta = true;
                self.version.effective = BETA_VERSION;
                self.version.parsed_as = BETA_VERSION;
                // The slice is selected before the permission is checked: a family without a
                // beta struct has no parser for `beta`.
                if !<F::History as VersionChain<F>>::HAS_BETA {
                    self.diagnostics.push(unsupported("beta", version_at()));
                    return None;
                }
                let refusal = match F::beta_permission(object, cx) {
                    BetaPermission::Always => None,
                    BetaPermission::BetaApisOrBaseGame => (!cx.beta_apis && !cx.is_base_game_pack)
                        .then_some(&core_codes::BETA_FORMAT_VERSION),
                    BetaPermission::BetaApis => {
                        (!cx.beta_apis).then_some(&core_codes::BETA_DOCUMENTS)
                    }
                };
                if let Some(code) = refusal {
                    self.diagnostics
                        .push(error(Code::from(code), version_at(), None));
                    return None;
                }
                Some((written, StructIndex::Beta))
            }
            Some(written) => {
                let text = written_text(&as_written);
                self.version.written = written.version;
                self.version.effective = written.version.unwrap_or(F::FLOOR);
                let index = self.stable::<F>(&written, &text, payload, &identity, cx)?;
                if written.prerelease {
                    self.diagnostics.push(Diagnostic::new(
                        core_codes::PRERELEASE_FORMAT_VERSION.with([text]),
                        Severity::Info,
                        Phase::Parse,
                        Location::new(None, None, version_at(), None),
                    ));
                }
                Some((written, index))
            }
        }
    }

    /// The envelope probe; a document that is no JSON object is rejected here.
    fn envelope<'a, F: Family>(&mut self, document: JsonSlice<'a>) -> Option<RootProbe<'a>> {
        match probe_envelope(document) {
            Ok(envelope) => Some(envelope),
            Err(failure) => {
                if is_blank(document.text) {
                    self.diagnostics.push(Diagnostic::new(
                        Code::from(&core_codes::EMPTY_DOCUMENT),
                        F::EMPTY_DOCUMENT,
                        Phase::Parse,
                        Location::new(None, None, JsonPath::new(), Some(span_of(document))),
                    ));
                    return None;
                }
                let message = failure.error.kind.to_string();
                let finding = if F::ENVELOPE.check == EnvelopeCheck::Never {
                    error(syntax(message), JsonPath::new(), Some(failure.span))
                } else {
                    error(
                        crate::diagnostic::codes::DOCUMENT_JSON.with([message.clone()]),
                        JsonPath::new(),
                        Some(failure.span),
                    )
                    .detail(DiagnosticDetail::Syntax(message))
                };
                self.diagnostics.push(finding);
                None
            }
        }
    }

    /// The stable struct `written` selects, or the rejection: below the floor, the line
    /// `Family::ENVELOPE.below_floor` names (`UseFormatVersion` or `NoParser`), or above every
    /// range ("no parser available").
    fn stable<F: Family>(
        &mut self,
        written: &FormatVersionInput,
        text: &str,
        payload: JsonSlice<'_>,
        identity: &Identity,
        cx: &LoadContext,
    ) -> Option<StructIndex> {
        let at = version_at();
        match select::<F>(written) {
            Selection::Struct(i) => {
                let count = <F::History as VersionChain<F>>::VERSIONS.len();
                let routed = F::route(payload, *written, cx).filter(|&r| r < count);
                Some(StructIndex::Stable(routed.unwrap_or(i)))
            }
            Selection::BelowFloor => {
                self.diagnostics.push(match F::ENVELOPE.below_floor {
                    // No slice below the first: the text above the engine has, the version
                    // rendered the same way.
                    BelowFloor::NoParser => unsupported(&no_parser_version(written, text), at),
                    _ => error(
                        core_codes::USE_FORMAT_VERSION.with([
                            F::NAME.to_owned(),
                            identity.identifier.clone().unwrap_or_default(),
                            F::FLOOR.to_string(),
                        ]),
                        at,
                        None,
                    ),
                });
                None
            }
            Selection::Unsupported => {
                // The version as the input parsed it (`1.26` is `1.26.0`, build metadata
                // dropped); a pre-release as written.
                self.diagnostics
                    .push(unsupported(&no_parser_version(written, text), at));
                None
            }
        }
    }
}

/// The envelope's version: `Ok(None)` when absent, `Err` when malformed (a string that is no
/// version, `"*"`, an array element above `u32`, any other JSON type).
fn read_version(version: &EnvelopeVersion) -> Result<Option<FormatVersionInput>, ()> {
    match version {
        EnvelopeVersion::Absent => Ok(None),
        EnvelopeVersion::String(text) => FormatVersionInput::parse(text).map(Some).ok_or(()),
        EnvelopeVersion::Array(parts) => FormatVersionInput::from_array(*parts).map(Some).ok_or(()),
        _ => Err(()),
    }
}

/// The version of a document-input family (`VersionInput::Document`): the first textual
/// `"format_version"` of the text, wherever it is, and the version as written for messages.
fn picked_version(text: &str) -> (Result<Option<FormatVersionInput>, ()>, EnvelopeVersion) {
    match FormatVersionInput::pick(text) {
        PickedVersion::Absent => (Ok(None), EnvelopeVersion::Absent),
        PickedVersion::Malformed => (Err(()), EnvelopeVersion::Malformed),
        PickedVersion::Version { input, written } => {
            (Ok(Some(input)), EnvelopeVersion::String(written))
        }
    }
}

/// The size check of a document input: above `MAX_JSON_DOCUMENT_BYTES` an envelope error
/// (code `0x10`), from `JSON_DOCUMENT_WARNING_BYTES` a warning. `true` when the document is
/// refused.
fn size_finding(len: usize, max: usize, told: &mut Diagnostics) -> bool {
    use crate::load::{JsonDocumentSize, json_document_size_against};
    let (code, severity) = match json_document_size_against(len, max) {
        JsonDocumentSize::Within => return false,
        // "JSON document is approaching the maximum allowed size": logged at level 2, and the
        // document is read.
        JsonDocumentSize::Approaching => (
            core_codes::DOCUMENT_NEARLY_TOO_LARGE.with([len.to_string(), max.to_string()]),
            Severity::Warning,
        ),
        JsonDocumentSize::TooLarge => (
            core_codes::DOCUMENT_TOO_LARGE.with([max.to_string()]),
            Severity::Error,
        ),
    };
    told.push(
        Diagnostic::new(
            code,
            severity,
            Phase::Parse,
            Location::new(None, None, JsonPath::new(), None),
        )
        .detail(DiagnosticDetail::Syntax(format!(
            "the document is {len} bytes long"
        ))),
    );
    severity == Severity::Error
}

/// What `RapidJSON` refuses of a text the envelope probe reads
/// ([`crate::json::document_json_refusal`]), told as the envelope tells a syntax error. `true`
/// when refused.
fn rapidjson_refuses<F: Family>(document: JsonSlice<'_>, told: &mut Diagnostics) -> bool {
    let Some(mut refusal) = crate::json::document_json_refusal(document) else {
        return false;
    };
    // A family whose envelope is never checked tells the reader's message alone.
    if F::ENVELOPE.check == EnvelopeCheck::Never
        && let Some(DiagnosticDetail::Syntax(message)) = refusal.detail.take().map(|detail| *detail)
    {
        refusal.code = syntax(message);
    }
    told.push(refusal);
    true
}

/// The version as written, for messages.
fn written_text(version: &EnvelopeVersion) -> String {
    match version {
        EnvelopeVersion::String(text) => text.clone(),
        EnvelopeVersion::Array([a, b, c]) => format!("{a}.{b}.{c}"),
        _ => String::new(),
    }
}

/// Whether the object `payload` has the member `key` (any value).
fn has_member(payload: JsonSlice<'_>, key: &str) -> bool {
    crate::json::from_str_borrowed_jsonc::<IndexMap<String, RawJson<'_>>>(payload.text)
        .is_ok_and(|members| members.contains_key(key))
}

/// An empty document (`EmptyDocument`): nothing but whitespace and comments, or `null`.
fn is_blank(text: &str) -> bool {
    let null = |text: &str| crate::json::from_str_jsonc::<Value>(text).is_ok_and(|v| v.is_null());
    // A trailing line comment ends at the added newline.
    null(text) || null(&format!("{text}\nnull"))
}

/// The path of the envelope's `format_version`.
fn version_at() -> JsonPath {
    let mut at = JsonPath::new();
    at.push(FORMAT_VERSION);
    at
}

/// An error-level parse finding.
fn error(code: Code, at: JsonPath, span: Option<Span>) -> Diagnostic {
    Diagnostic::new(
        code,
        Severity::Error,
        Phase::Parse,
        Location::new(None, None, at, span),
    )
}

#[cfg(test)]
mod tests {
    use super::size_finding;
    use crate::diagnostic::codes as core_codes;
    use crate::diagnostic::{Diagnostics, Severity};
    use crate::load::{JSON_DOCUMENT_WARNING_BYTES, MAX_JSON_DOCUMENT_BYTES};

    /// The document input's size check goes by length alone: nothing below 96 MiB, the
    /// size warning up to the limit, the size error (the document refused) above it.
    #[test]
    fn a_document_input_sizes_the_text_before_reading_it() {
        let mut told = Diagnostics::default();
        let max = MAX_JSON_DOCUMENT_BYTES;
        assert!(!size_finding(
            JSON_DOCUMENT_WARNING_BYTES - 1,
            max,
            &mut told
        ));
        assert!(told.is_empty());
        assert!(!size_finding(MAX_JSON_DOCUMENT_BYTES, max, &mut told));
        let warning = told.iter().next().expect("the size warning");
        assert_eq!(warning.severity, Severity::Warning);
        let limit = MAX_JSON_DOCUMENT_BYTES.to_string();
        assert_eq!(
            warning.code,
            core_codes::DOCUMENT_NEARLY_TOO_LARGE.with([limit.clone(), limit])
        );
        let mut told = Diagnostics::default();
        assert!(size_finding(MAX_JSON_DOCUMENT_BYTES + 1, max, &mut told));
        let error = told.first_error().expect("the size error");
        assert_eq!(
            error.code.to_string(),
            "Minecraft Document JSON exceeds the maximum size of 134217728 bytes"
        );
    }
}
