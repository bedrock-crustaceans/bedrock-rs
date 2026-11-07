//! Parsing a feature payload the way vanilla's strict schema validates it: typed from its slice
//! of the text, and every unknown member, wrong type or missing required member rejects the
//! file: the client validates with unknown children not allowed. And the loader stage before
//! it: the identifier the file claims, the gates of its feature type, and the load, parse and
//! upgrade of one file by the family of its payload key.

use facet::Facet;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, CodeDef, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, Severity,
};
use bedrock_addon_core::json::{
    JsonSlice, NullRule, ScalarFailure, SliceError, Value, parse_with_rules, unknown_members,
};
use bedrock_addon_core::versioning::ParseCx;

use crate::codes;
use crate::family::{Fam, FeatureType, TypeFn, V1_13_0, with_type};
use crate::{Feature, history};
use bedrock_addon_core::document::{Document, Origin, Rejected};
use bedrock_addon_core::id::FeatureId;
use bedrock_addon_core::json::{PrepareError, Prepared};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersionInput;
use bedrock_addon_core::versioning::{
    Loaded, Parsed, file_version, finish, load, parse_as_written,
};

/// The last member name of `path` as the file has it (`"y"` for
/// `minecraft:scatter_feature/distribution/y`); an element's number for a path that ends at
/// an array element.
pub(crate) fn last_key(path: &JsonPath) -> String {
    match path.segments().last() {
        Some(bedrock_addon_core::diagnostic::JsonPathSegment::Key(key)) => key.to_owned(),
        Some(bedrock_addon_core::diagnostic::JsonPathSegment::Index(index)) => index.to_string(),
        None => String::new(),
    }
}

/// `UnrecognizedField` (error) with vanilla's text, for the member at `at`.
pub(crate) fn not_valid_here(at: JsonPath) -> Diagnostic {
    let key = last_key(&at);
    Diagnostic::of(core_codes::CHILD_NOT_VALID.with([key]))
        .severity(Severity::Error)
        .at(at)
}

/// `MissingRequiredField` (error) with vanilla's text, for the member at `at`.
pub(crate) fn required_child(at: JsonPath) -> Diagnostic {
    let key = last_key(&at);
    Diagnostic::of(core_codes::REQUIRED_CHILD.with([key])).at(at)
}

/// Parses `T` from `payload` strictly: a failed typed parse is one error at the failing member
/// (an unknown enum name is `InvalidValue`, as vanilla's enum nodes reject it), and every member
/// the struct does not take is an `UnrecognizedField` error. `None` when anything failed.
pub(crate) fn parse_strict<'a, T: Facet<'a>>(
    payload: JsonSlice<'a>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<T> {
    parse_strict_with(payload, cx, d, &[])
}

/// [`parse_strict`] with the setters' texts for members whose value is a name: an unknown name
/// of the member `key` is the definition beside it (vanilla's "Bad value for …" messages).
pub(crate) fn parse_strict_with<'a, T: Facet<'a>>(
    payload: JsonSlice<'a>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
    texts: &[(&str, &'static CodeDef<0>)],
) -> Option<T> {
    // The null rule of the selected version, which the family declares (`ENVELOPE.input` is
    // `JsonCpp`): under rule N1 a member written as `null` is absent.
    let absent = cx.load.null_rule == Some(NullRule::Absent);
    // The scalars are read by the family's reading (`family::READING`), the strict rule.
    let null = absent.then_some(NullRule::Absent);
    let parsed = match parse_with_rules::<T>(payload, null, crate::family::READING.scalar_rules()) {
        Ok(read) => {
            for told in &read.told {
                d.push(told.finding(&cx.at));
            }
            Some(read.value)
        }
        // A scalar no arm of an untagged enum takes: a type error at the enum.
        Err(ScalarFailure::Refused(misread)) => {
            d.push(misread.spanned_finding(&cx.at));
            None
        }
        Err(ScalarFailure::Read(error)) => {
            let mut finding = slice_error(error, &cx.at);
            if finding.code.is(&core_codes::INVALID_VALUE) {
                let key = last_key(&finding.location.json_path);
                if let Some((_, message)) = texts.iter().find(|(k, _)| *k == key) {
                    finding.code = Code::from(*message);
                }
            }
            d.push(finding);
            None
        }
    };
    // Vanilla's validation reports every unknown member, also beside another failure.
    if let Ok(json) = bedrock_addon_core::json::from_str_jsonc::<Value>(payload.text) {
        if absent {
            // The members read as absent, which the write-back writes as their defaults
            // told here once, the only N1 read of a feature payload.
            let found = bedrock_addon_core::json::null_members(T::SHAPE, &json);
            bedrock_addon_core::json::tell_null_members(&found, &cx.at, d);
        }
        let unknown = unknown_members(T::SHAPE, &json);
        for (path, _) in &unknown {
            d.push(not_valid_here(cx.at.join(path)));
        }
        if !unknown.is_empty() {
            return None;
        }
    }
    parsed
}

/// The finding of a failed typed parse of `slice`, at `base` followed by the failing member.
fn slice_error(error: SliceError, base: &JsonPath) -> Diagnostic {
    error.into_diagnostic(base, |code, error| {
        let at = || base.join(error.relative_path());
        match code.id() {
            // The client's enum members reject an unknown name as a value.
            "core/invalid_enum_value" => Code::from(&core_codes::INVALID_VALUE),
            "core/missing_required_field" => core_codes::REQUIRED_CHILD.with([last_key(&at())]),
            "core/unrecognized_field" => core_codes::CHILD_NOT_VALID.with([last_key(&at())]),
            _ => code,
        }
    })
}

// ---- the loader stage -------------------------------------------------------------------------

/// The `description.identifier` string of the first root member that has one.
fn identifier_in(root: &Value) -> Option<String> {
    root.as_object()?.iter().find_map(|(_, member)| {
        let identifier = member
            .as_object()?
            .get("description")?
            .as_object()?
            .get("identifier")?
            .as_string()?;
        Some(identifier.as_str().to_owned())
    })
}

/// The payload key: the first root member other than `format_version`.
fn payload_key(root: &Value) -> String {
    root.as_object()
        .and_then(|object| {
            object
                .keys()
                .find(|key| key.as_str() != "format_version")
                .map(|key| key.as_str().to_owned())
        })
        .unwrap_or_default()
}

/// What the loader stage gives a type's parse: the text, its identifier and context.
struct Stage<'a> {
    text: &'a str,
    key: &'a str,
    cx: &'a LoadContext,
}

/// The checks the client makes before the schema (the internal-type gate and the engine gate
/// of the type) and the version floor, for type `T`; the findings when the document stops
/// here.
fn type_checks<T: FeatureType>(stage: &Stage<'_>) -> Result<(), Diagnostics> {
    let mut d = Diagnostics::default();
    let mut at = JsonPath::new();
    if T::SINCE_ENGINE.is_some_and(|since| stage.cx.engine_version < since) {
        // The engine has not registered the type.
        d.push(Diagnostic::of(core_codes::UNKNOWN_PAYLOAD_KEY.with([stage.key.to_owned()])).at(at));
        return Err(d);
    }
    if T::INTERNAL && !stage.cx.is_base_game_pack {
        // An internal type is for the game's own packs.
        at.push(T::KEY);
        d.push(Diagnostic::of(codes::INTERNAL_TYPE.with([stage.key])).at(at));
        return Err(d);
    }
    let version = file_version(JsonSlice::new(stage.text));
    // A type registered at a later format version than the initial release (multi_block and
    // multipart_block_column) is in no schema below its floor: the client says "Could not find
    // registered feature type that matches name '%s'".
    if T::FLOOR > V1_13_0
        && let Some(version) = version
        && version < T::FLOOR
    {
        d.push(Diagnostic::of(core_codes::UNKNOWN_PAYLOAD_KEY.with([stage.key.to_owned()])).at(at));
        return Err(d);
    }
    // Below the floor no parser's range matches: the client's "no parser available".
    if let Some(version) = version
        && version < T::FLOOR
    {
        at.push("format_version");
        d.push(
            Diagnostic::of(core_codes::NO_PARSER_FOR_VERSION.with([version.to_string()])).at(at),
        );
        return Err(d);
    }
    Ok(())
}

/// The full load of a document of one type: the outcome as a `Feature`.
struct LoadType<'a>(Stage<'a>);

impl
    TypeFn<(
        Option<Feature>,
        bedrock_addon_core::document::VersionInfo,
        Diagnostics,
    )> for LoadType<'_>
{
    fn call<T: FeatureType>(
        self,
    ) -> (
        Option<Feature>,
        bedrock_addon_core::document::VersionInfo,
        Diagnostics,
    ) {
        let stage = self.0;
        if let Err(d) = type_checks::<T>(&stage) {
            return (None, no_version(), d);
        }
        let outcome = load::<Fam<T>>(JsonSlice::new(stage.text), stage.cx);
        let value = match outcome.value {
            Some(Loaded::Stable(value)) => Some(value.into_feature()),
            _ => None,
        };
        (value, outcome.version, outcome.diagnostics)
    }
}

/// The parse of a document of one type without the upgrade.
struct ParseType<'a>(Stage<'a>);

impl
    TypeFn<(
        Option<history::FeatureHistory>,
        bedrock_addon_core::document::VersionInfo,
        Diagnostics,
    )> for ParseType<'_>
{
    fn call<T: FeatureType>(
        self,
    ) -> (
        Option<history::FeatureHistory>,
        bedrock_addon_core::document::VersionInfo,
        Diagnostics,
    ) {
        let stage = self.0;
        if let Err(d) = type_checks::<T>(&stage) {
            return (None, no_version(), d);
        }
        let parsed = parse_as_written::<Fam<T>>(JsonSlice::new(stage.text), stage.cx);
        (
            parsed.value.map(T::to_history),
            parsed.version,
            parsed.diagnostics,
        )
    }
}

/// The version information of a document that stopped before its version was read.
fn no_version() -> bedrock_addon_core::document::VersionInfo {
    bedrock_addon_core::document::VersionInfo::new(None, false, V1_13_0, V1_13_0, false)
}

/// Whose definition a feature file is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FileSource {
    /// A pack's file.
    Pack,
    /// One of the app's own definitions: its identifier must be in the `minecraft` namespace.
    BuiltIn,
}

/// Whether a duplicate identifier is reported with vanilla's text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DuplicateText {
    /// An information without text.
    Silent,
    /// An error with vanilla's text.
    Vanilla,
}

/// The loader stage up to the identifier (valid JSON, an identifier that is present, not
/// empty and namespaced, and the built-in namespace rule): the text, its root and its
/// identifier; or the findings of a file that stops before the identifier map, so it claims no
/// identifier.
fn identify<'b>(
    prepared: &'b Result<Prepared<'b>, PrepareError>,
    path: &PackPath,
    source: FileSource,
) -> Result<(&'b str, Value, FeatureId), Diagnostics> {
    let file = path.to_string();
    let invalid = |message: String| {
        let mut d = Diagnostics::default();
        d.push(
            Diagnostic::of(codes::INVALID_JSON.with([file.as_str()]))
                .detail(DiagnosticDetail::Syntax(message)),
        );
        d
    };
    let text = prepared
        .as_ref()
        .map(Prepared::as_str)
        .map_err(|_| invalid("not UTF-8 JSON within the nesting limit".to_owned()))?;
    // The identifier is read from the copy of a doubled member that stands, as the parse
    // reads it.
    let root = bedrock_addon_core::json::from_str_jsonc::<Value>(
        &bedrock_addon_core::json::last_wins(text),
    )
    .map_err(|e| invalid(e.to_string()))?;
    let one = |code: Code| {
        let mut d = Diagnostics::default();
        d.push(Diagnostic::of(code));
        d
    };
    let Some(identifier) = identifier_in(&root) else {
        return Err(one(codes::MISSING_IDENTIFIER.with([file.as_str()])));
    };
    if identifier.is_empty() {
        return Err(one(codes::EMPTY_IDENTIFIER.with([file.as_str()])));
    }
    let Some((namespace, _)) = identifier.split_once(':') else {
        return Err(one(codes::NOT_NAMESPACED.with([identifier.as_str()])));
    };
    if source == FileSource::BuiltIn && namespace != "minecraft" {
        return Err(one(codes::NOT_MINECRAFT.with([identifier.as_str()])));
    }
    Ok((text, root, FeatureId::parse(identifier)))
}

/// Loads one feature file, read from `origin`: the identifier it claims (when the loader stage
/// read one) and the document or its rejection. `source` says whose definition the file is.
pub(crate) fn parse_file(
    origin: Origin,
    bytes: &[u8],
    cx: &LoadContext,
    source: FileSource,
) -> (Option<FeatureId>, Result<Document<Feature>, Rejected>) {
    let prepared = Prepared::new(bytes);
    let (text, root, identifier) = match identify(&prepared, &origin.path, source) {
        Ok(found) => found,
        Err(d) => return (None, Err(Rejected::located(origin, d))),
    };
    let key = payload_key(&root);
    let stage = Stage {
        text,
        key: &key,
        cx,
    };
    let Some((value, version, mut diagnostics)) = with_type(&key, LoadType(stage)) else {
        let mut d = Diagnostics::default();
        d.push(Diagnostic::of(
            core_codes::UNKNOWN_PAYLOAD_KEY.with([key.clone()]),
        ));
        return (Some(identifier), Err(Rejected::located(origin, d)));
    };
    diagnostics.locate(&origin);
    let result = match value {
        Some(feature) => Ok(Document::new(feature, origin, version, diagnostics)),
        None => Err(Rejected::new(origin, diagnostics)),
    };
    (Some(identifier), result)
}

/// [`Feature::parse_as_written`] for a file of `pack`.
pub(crate) fn parse_file_as_written(
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<history::FeatureHistory>, Rejected> {
    let origin = Origin::file(PackRef::app_data(), PackSide::Behavior, path.clone());
    let prepared = Prepared::new(bytes);
    let (text, root, _) = match identify(&prepared, path, FileSource::Pack) {
        Ok(found) => found,
        Err(d) => return Err(Rejected::located(origin, d)),
    };
    let key = payload_key(&root);
    let stage = Stage {
        text,
        key: &key,
        cx,
    };
    let Some((value, version, mut diagnostics)) = with_type(&key, ParseType(stage)) else {
        let mut d = Diagnostics::default();
        d.push(Diagnostic::of(
            core_codes::UNKNOWN_PAYLOAD_KEY.with([key.clone()]),
        ));
        return Err(Rejected::located(origin, d));
    };
    diagnostics.locate(&origin);
    match value {
        Some(value) => Ok(Document::new(value, origin, version, diagnostics)),
        None => Err(Rejected::new(origin, diagnostics)),
    }
}

/// The upgrade of a document `parse_as_written` gave.
struct UpgradeType<'a> {
    any: history::FeatureHistory,
    version: bedrock_addon_core::document::VersionInfo,
    diagnostics: Diagnostics,
    cx: &'a LoadContext,
}

impl
    TypeFn<
        Result<
            (
                Feature,
                bedrock_addon_core::document::VersionInfo,
                Diagnostics,
            ),
            Diagnostics,
        >,
    > for UpgradeType<'_>
{
    fn call<T: FeatureType>(
        self,
    ) -> Result<
        (
            Feature,
            bedrock_addon_core::document::VersionInfo,
            Diagnostics,
        ),
        Diagnostics,
    > {
        let identifier = self.any.identifier().as_str().to_owned();
        let Ok(chain) = T::from_history(self.any) else {
            return Err(self.diagnostics);
        };
        let at = JsonPath::new().child(T::KEY);
        let written = FormatVersionInput {
            version: Some(self.version.effective),
            beta_literal: false,
            prerelease: false,
        };
        let identity = bedrock_addon_core::versioning::Identity {
            identifier: Some(identifier),
            runtime_identifier: None,
        };
        let parsed =
            Parsed::<Fam<T>>::from_value(Some(chain), self.version, written, at, self.diagnostics)
                .with_identity(identity);
        let outcome = finish(parsed, self.cx);
        match outcome.value {
            Some(Loaded::Stable(value)) => {
                Ok((value.into_feature(), outcome.version, outcome.diagnostics))
            }
            _ => Err(outcome.diagnostics),
        }
    }
}

/// Rebuilds the parse stage from the document and finishes it.
pub(crate) fn upgrade_document(
    document: Document<history::FeatureHistory>,
    cx: &LoadContext,
) -> Result<Document<Feature>, Rejected> {
    let (any, origin, version, diagnostics) = document.into_parsed_parts();
    let key = any.type_key();
    let upgrade = UpgradeType {
        any,
        version,
        diagnostics: diagnostics.clone(),
        cx,
    };
    match with_type(key, upgrade) {
        Some(Ok((feature, version, mut diagnostics))) => {
            diagnostics.locate(&origin);
            Ok(Document::new(feature, origin, version, diagnostics))
        }
        Some(Err(diagnostics)) => Err(Rejected::located(origin, diagnostics)),
        None => Err(Rejected::new(origin, diagnostics)),
    }
}
