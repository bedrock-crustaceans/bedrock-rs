//! What reading an animation file and an animation controller file share: the text check, the
//! `format_version` member, the parsed-file record, the reads of the values both files hold
//! (events of one or many, time-keyed members, event strings), and the document a parsed file
//! becomes.

use facet::Facet;
use indexmap::IndexMap;

use super::event::command_version;
use super::types::EventString;
use super::walk::TIMES;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticDetail, Diagnostics};
use bedrock_addon_core::document::{Document, Origin, Rejected, VersionInfo};
use bedrock_addon_core::json::kit::{Expected, Node, Read};
use bedrock_addon_core::json::parse::{Options, Parsed as Text, parse};
use bedrock_addon_core::json::{MAX_JSON_DEPTH, Winner, check_depth, strip_bom};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackSide, PackType};
use bedrock_addon_core::schema::{FromJson, read_each, read_keyed};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, SemVersion};
use bedrock_addon_shared_shapes::OneOrMany;

/// A parsed file: the value when it loads, the version as written, every finding.
#[derive(Clone, Debug, PartialEq)]
pub struct Parsed<T> {
    /// The value, when the file loads.
    pub value: Option<T>,
    /// The `format_version` as written, when the file has one.
    pub written: Option<SemVersion>,
    /// Every finding of the read.
    pub diagnostics: Diagnostics,
}

/// The text of a file after the BOM, the depth check and the JSON syntax check, read with its
/// spans for the schema read; the finding when it is not readable (vanilla "parse error:\n%s").
///
/// # Errors
///
/// The `MalformedInput` finding (boxed) when the text is nested deeper than `MAX_JSON_DEPTH`, is
/// not UTF-8, or is not valid JSON.
pub fn text_of(bytes: &[u8]) -> Result<Text<'_>, Box<Diagnostic>> {
    let malformed = |message: String| {
        Box::new(
            Diagnostic::of(core_codes::PARSE_ERROR.with([message.as_str()]))
                .detail(DiagnosticDetail::Syntax(message)),
        )
    };
    let bytes = strip_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH).map_err(|_| malformed("nesting too deep".to_owned()))?;
    let text = std::str::from_utf8(bytes).map_err(|e| malformed(e.to_string()))?;
    // `Json::Reader::parse` of the whole text first: a syntax error anywhere drops the file,
    // in the words of the door's reader.
    // Of a key written twice, the first place and the last value, as the walker of the game's
    // schema reads an object.
    parse(text, Options::winner(Winner::LastInPlace))
        .map_err(|fault| malformed(fault.to_error().to_string()))
}

/// A `format_version` as written, as a schema's `SemVersion` node reads it.
#[derive(Clone, Debug, Facet)]
#[facet(opaque)]
pub struct Written(pub SemVersion);

bedrock_addon_core::impl_schema_leaf!(Written);
bedrock_addon_core::impl_no_molang!(Written);

/// A `format_version` member of a schema: a version string (`NoWildcards`), at least `minimum`.
pub fn format_version(
    node: &Node<'_>,
    read: &mut Read<'_>,
    minimum: &SemVersion,
) -> Option<Written> {
    let written = node.value().as_string().map(facet_value::VString::as_str);
    let version = written
        .filter(|t| *t != "beta")
        .and_then(SemVersion::parse)
        .filter(|v| !v.is_any());
    let Some(version) = version else {
        read.wrong_accepting(node, "SemVersion");
        return None;
    };
    if version < *minimum {
        let code = bedrock_addon_core::diagnostic::cereal::version_below_min(
            written.unwrap_or(""),
            &minimum.to_string(),
        );
        read.error(node, code);
        return None;
    }
    Some(Written(version))
}

/// The version input of a parsed `SemVersion`.
pub fn input_of(version: &SemVersion) -> FormatVersionInput {
    FormatVersionInput {
        version: Some(version.format_version()),
        beta_literal: false,
        prerelease: version.is_pre_release(),
    }
}

/// One value or an array of them (`particle_effects`, `sound_effects`, `timeline`): an array is
/// read element by element, a value of `T`'s kind as `T`; another value is a wrong type
/// accepting both.
pub fn one_or_many<T: FromJson>(node: &Node<'_>, read: &mut Read<'_>) -> Option<OneOrMany<T>> {
    let value = node.value();
    if value.is_array() {
        return read_each(node, read, T::from_json).map(OneOrMany::Many);
    }
    let kind = T::EXPECTED;
    let of_kind = match kind {
        Some(Expected::Object) => value.is_object(),
        Some(Expected::String) => value.is_string(),
        _ => true,
    };
    if !of_kind {
        let word = kind.map_or("value", Expected::word);
        read.wrong_accepting(node, &format!("{word}, array"));
        return None;
    }
    T::from_json(node, read).map(OneOrMany::One)
}

/// Members by their time key, each one value or an array of them.
pub fn timed<T: FromJson>(
    node: &Node<'_>,
    read: &mut Read<'_>,
) -> Option<IndexMap<String, OneOrMany<T>>> {
    read_keyed(node, read, &TIMES, one_or_many::<T>)
}

/// A bool whose allowed-value set is `{false}` (`bind_to_actor`).
pub fn only_false(node: &Node<'_>, read: &mut Read<'_>) -> Option<bool> {
    let value = bool::from_json(node, read)?;
    if value {
        read.error(node, core_codes::VALUE_NOT_ALLOWED.with(["true"]));
        return None;
    }
    Some(value)
}

/// A string, split the way the game splits an event, with the pack's command version.
impl FromJson for EventString {
    const EXPECTED: Option<Expected> = Some(Expected::String);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let text = read.string(node)?;
        let pack = read
            .load_context()
            .and_then(|cx| cx.min_engine_version.as_ref());
        Some(Self::classify(text, command_version(pack)))
    }
}

bedrock_addon_core::impl_schema_leaf!(EventString);

/// The side a context's pack is on (a resource pack is the client).
pub fn side_of(cx: &LoadContext) -> PackSide {
    if cx.pack_type == PackType::Resource {
        PackSide::Resource
    } else {
        PackSide::Behavior
    }
}

/// Gives every finding the file's pack and path, pushed one by one: the places the findings
/// held are not carried over.
// TODO(phase 1 step 4): this loop becomes the existing `Diagnostics::locate`, as core's
// `unversioned` one does. That switch is behaviour-bearing: the repush here drops the moves
// and places, `locate` keeps them in place.
fn locate(diagnostics: Diagnostics, origin: &Origin) -> Diagnostics {
    let mut out = Diagnostics::default();
    for mut finding in diagnostics {
        finding.location.pack = Some(origin.pack.clone());
        finding.location.path = Some(origin.path.clone());
        out.push(finding);
    }
    out
}

/// The version record of a file whose `format_version` was written as `written`, parsed as
/// `parsed_as`, and whether it was upgraded to get there.
pub fn version_info(written: &SemVersion, parsed_as: FormatVersion, upgraded: bool) -> VersionInfo {
    let written = written.format_version();
    VersionInfo::new(Some(written), false, written, parsed_as, upgraded)
}

/// One file's result as a document of the whole file.
///
/// # Errors
///
/// `Rejected` (with the file's located diagnostics) when the parse produced no value or the file's
/// `format_version` was not read.
pub fn whole<T>(
    parsed: Parsed<T>,
    origin: Origin,
    parsed_as: impl FnOnce(&SemVersion) -> (FormatVersion, bool),
) -> Result<Document<T>, Rejected> {
    let diagnostics = locate(parsed.diagnostics, &origin);
    match (parsed.value, parsed.written) {
        (Some(value), Some(written)) => {
            let (as_, upgraded) = parsed_as(&written);
            Ok(Document::new(
                value,
                origin,
                version_info(&written, as_, upgraded),
                diagnostics,
            ))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}
