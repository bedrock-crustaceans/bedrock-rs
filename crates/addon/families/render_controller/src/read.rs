//! Reading render controller files: the case fold, the one strict schema walked over the text
//! (file stage), then the per-controller stage, the Molang passes and the crate's own findings.

use std::collections::HashMap;

use facet::Facet;
use facet_value::VObject;
use indexmap::{IndexMap, IndexSet};

use super::codes;
use super::fold::{Folded, fold};
use super::molang::{MolangRun, run};
use super::{BonePattern, RenderController, RenderControllerFile};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, JsonPathSegment, Phase, Severity,
    Span,
};
use bedrock_addon_core::json::kit::{Lossy, MemberOrder, Node, Profile, Read, Texts};
use bedrock_addon_core::json::parse::{Options, Parsed as Text, parse};
use bedrock_addon_core::json::{MAX_JSON_DEPTH, Value, Winner, check_depth};
use bedrock_addon_core::json::{PrepareError, Prepared};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::{Molang, MolangSource, RawVersion};
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::schema::{
    self, FromJson, KeyPattern, Keys, Schema, read_document, read_each, read_keyed,
};
use bedrock_addon_core::version::{ENGINE_VERSION, FormatVersionInput, SemVersion};
use bedrock_addon_core::versioning::Reading;

/// The schema's `format_version` minimum and the family's floor.
pub(crate) const V1_8_0: SemVersion = SemVersion::new(1, 8, 0);
/// The version node's default maximum, which the schema never overrides.
pub(crate) const MAXIMUM: SemVersion = SemVersion::new(65535, 0, 0);

/// The profile of that schema: the node tree's, names as written, the members of an object in
/// sorted key order (jsoncpp's `std::map`), and a lossy float told.
pub(crate) const PROFILE: Profile = Profile::TREE
    .with_members(MemberOrder::Sorted)
    .with_lossy(Lossy::Told);

/// The controller keys: `controller\.render\.[a-zA-Z0-9_.:-]+`, at least one.
const CONTROLLERS: Keys = Keys::new(KeyPattern::Prefixed {
    prefix: "controller.render.",
    extra: b"_.:-",
    regex: r"controller\.render\.[a-zA-Z0-9_.:-]+",
})
.required();
/// The array names of every `arrays` kind: `array\.[a-zA-Z0-9_.:-]+`, at least one.
const ARRAYS: Keys = Keys::new(KeyPattern::Prefixed {
    prefix: "array.",
    extra: b"_.:-",
    regex: r"array\.[a-zA-Z0-9_.:-]+",
})
.required();
/// The bone patterns of one element: `[a-zA-Z0-9_.:*]+`, at least one.
const BONES: Keys = Keys::new(KeyPattern::Charset {
    extra: b"_.:*",
    regex: "[a-zA-Z0-9_.:*]+",
})
.required();

/// What reading one file gave.
pub(crate) struct Parsed {
    /// The file with the controllers that loaded; `None` when the file stage failed.
    pub(crate) file: Option<RenderControllerFile>,
    /// The controllers the controller stage dropped, each with its findings.
    pub(crate) dropped: Vec<(String, Diagnostics)>,
    /// The version as written, when the file stage read one.
    pub(crate) written: Option<SemVersion>,
    /// The file's findings, those of the controllers that loaded included.
    pub(crate) diagnostics: Diagnostics,
}

impl Parsed {
    fn failed(diagnostics: Diagnostics, written: Option<SemVersion>) -> Self {
        Self {
            file: None,
            dropped: Vec::new(),
            written,
            diagnostics,
        }
    }
}

/// Reads one render controller file at `path` of a pack with context `cx`.
pub(crate) fn parse_file(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Parsed {
    // A leading byte-order mark is blanked once, here, for both readings below: every span
    // is an offset into the file as written.
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let mut parsed = parse_folded(path, bytes, cx);
    // The file is read folded, so a finding's path spells a key in lower case; the
    // file's own spelling is given back: the path names the member the file has.
    let folded_key = |d: &Diagnostics| {
        d.iter().any(|finding| {
            finding
                .location
                .json_path
                .segments()
                .any(|segment| matches!(segment, JsonPathSegment::Key(_)))
        })
    };
    if (folded_key(&parsed.diagnostics) || parsed.dropped.iter().any(|(_, d)| folded_key(d)))
        && bytes.iter().any(u8::is_ascii_uppercase)
        && let Ok(written) = bedrock_addon_core::json::parse::parse_bytes(bytes, Options::default())
    {
        let mut spelled = Spelled::new();
        parsed.diagnostics = as_spelled(
            std::mem::take(&mut parsed.diagnostics),
            &written,
            &mut spelled,
        );
        for (_, findings) in &mut parsed.dropped {
            *findings = as_spelled(std::mem::take(findings), &written, &mut spelled);
        }
    }
    parsed
}

/// The keys of the objects of a file as written, by their folded spelling (the last of two
/// keys the fold makes equal, as the reader keeps the last): one table an object, built when
/// the first finding passes through it and shared by every finding of the file.
type Spelled<'v> = HashMap<*const VObject, HashMap<String, &'v str>>;

/// `findings` with every key of a path spelled as `written` has it (the keys of the folded
/// text differ from the file's in the case of ASCII letters alone). A path that leaves
/// `written` keeps its spelling from there on.
fn as_spelled<'v>(
    findings: Diagnostics,
    written: &'v Value,
    spelled: &mut Spelled<'v>,
) -> Diagnostics {
    findings
        .into_iter()
        .map(|mut finding| {
            let mut path = JsonPath::new();
            let mut node = Some(written);
            for segment in finding.location.json_path.segments() {
                match segment {
                    JsonPathSegment::Index(index) => {
                        path.push_index(index);
                        node = node.and_then(Value::as_array).and_then(|a| a.get(index));
                    }
                    JsonPathSegment::Key(key) => {
                        let object = node.and_then(Value::as_object);
                        let name = object.and_then(|object| {
                            spelled
                                .entry(std::ptr::from_ref(object))
                                .or_insert_with(|| {
                                    object
                                        .keys()
                                        .map(|name| {
                                            (name.as_str().to_ascii_lowercase(), name.as_str())
                                        })
                                        .collect()
                                })
                                .get(key)
                                .copied()
                        });
                        path.push(name.unwrap_or(key));
                        node = name.and_then(|name| object.and_then(|object| object.get(name)));
                    }
                }
            }
            finding.location.json_path = path;
            finding
        })
        .collect()
}

/// [`parse_file`], with the findings at the paths of the folded text; `bytes` are the file with
/// its byte-order mark blanked.
#[allow(
    clippy::too_many_lines,
    reason = "the load stages, in the game's order"
)]
fn parse_folded(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Parsed {
    let malformed = |message: String| {
        let mut diagnostics = Diagnostics::default();
        diagnostics.push(
            Diagnostic::of(codes::ERROR_PARSING.with([path.as_str(), &message]))
                .detail(DiagnosticDetail::Syntax(message)),
        );
        Parsed::failed(diagnostics, None)
    };
    // The nesting before the encoding, each in the loader's words.
    if check_depth(bytes, MAX_JSON_DEPTH).is_err() {
        return malformed("nesting too deep".to_owned());
    }
    let prepared = match Prepared::new(bytes) {
        Ok(prepared) => prepared,
        Err(PrepareError::NotUtf8(e)) => return malformed(e.to_string()),
        Err(_) => return malformed("nesting too deep".to_owned()),
    };
    let original = prepared.as_str();
    // the fold runs on the raw text, before the JSON reader sees it.
    let folded = fold(original);
    // The whole text is parsed first: a syntax error anywhere drops the file. Of a key written
    // twice (also two keys the fold made equal) the last value stands.
    let text = match parse(&folded.text, Options::winner(Winner::Last)) {
        Ok(text) => text,
        Err(fault) => return malformed(fault.to_error().to_string()),
    };

    let mut diagnostics = Diagnostics::default();
    let mut read = Read::new(Reading::TREE, &mut diagnostics)
        .profile(PROFILE)
        .texts(Texts::TREE);
    let document = read_root(&text, &mut read);
    drop(read);
    let Some((written, controllers)) = document.filter(|_| !diagnostics.has_errors()) else {
        folding_hints(&mut diagnostics, &folded, original);
        return Parsed::failed(diagnostics, None);
    };

    let at_version = || JsonPath::of_keys(["format_version"]);
    unresolvable_names(
        marked(&controllers, &text),
        &folded,
        original,
        &mut diagnostics,
    );
    if written.is_pre_release() {
        diagnostics.push(
            Diagnostic::of(core_codes::PRERELEASE_FORMAT_VERSION.with([written.to_string()]))
                .at(at_version()),
        );
    }
    if written.format_version() > cx.engine_version {
        // The game compares the version with nothing but the schema's bounds.
        diagnostics.push(
            Diagnostic::of(
                core_codes::FORMAT_VERSION_ABOVE_ENGINE
                    .with([written.to_string(), ENGINE_VERSION.to_string()]),
            )
            .at(at_version()),
        );
    }

    let numeric_materials = numeric_materials(&text.value);
    let without_textures: Vec<String> = controllers
        .iter()
        .filter(|(_, controller)| controller.textures.is_empty())
        .map(|(key, _)| key.clone())
        .collect();
    let mut file = RenderControllerFile {
        format_version: written.format_version(),
        render_controllers: controllers,
        dropped: IndexMap::new(),
    };
    // The Molang passes: versions, then the compile check with each member's failure scope.
    let outcome = run(
        &mut file,
        &MolangRun {
            load: cx,
            written: FormatVersionInput {
                version: Some(written.format_version()),
                beta_literal: false,
                prerelease: written.is_pre_release(),
            },
            numeric_materials: &numeric_materials,
        },
        &mut diagnostics,
    );
    if outcome.file_rejected {
        return Parsed::failed(diagnostics, Some(written));
    }

    // The controller stage: `RenderController::parse` returned an error for these.
    let mut failed: IndexSet<String> = outcome.dropped;
    for key in without_textures {
        diagnostics.push(
            Diagnostic::of(&codes::MUST_CONTAIN_TEXTURES)
                .at(controller_path(&key, Some("textures"))),
        );
        failed.insert(key);
    }
    for (key, controller) in &file.render_controllers {
        if !failed.contains(key) {
            super::validate::array_references(key, controller, &mut diagnostics);
        }
    }
    // The dropped controllers stay in the file, apart, so that it is written back whole.
    for (key, controller) in std::mem::take(&mut file.render_controllers) {
        if failed.contains(&key) {
            file.dropped.insert(key, controller);
        } else {
            file.render_controllers.insert(key, controller);
        }
    }
    // One pass over the findings, however many controllers were dropped.
    let (mut by_key, diagnostics) = group_by_controller(diagnostics, |key| failed.contains(key));
    let dropped = failed
        .into_iter()
        .map(|key| {
            let mine = by_key.swap_remove(&key).unwrap_or_default();
            (key, mine)
        })
        .collect();
    Parsed {
        file: Some(file),
        dropped,
        written: Some(written),
        diagnostics,
    }
}

/// `render_controllers/<key>[/<member>]`.
pub(crate) fn controller_path(key: &str, member: Option<&str>) -> JsonPath {
    let mut at = JsonPath::of_keys(["render_controllers", key]);
    if let Some(member) = member {
        at.push(member);
    }
    at
}

/// The controller key a finding's path is below, if it is below one.
pub(crate) fn controller_of(path: &JsonPath) -> Option<&str> {
    let mut segments = path.segments();
    match (segments.next(), segments.next()) {
        (Some(JsonPathSegment::Key("render_controllers")), Some(JsonPathSegment::Key(key))) => {
            Some(key)
        }
        _ => None,
    }
}

/// Splits the findings in one pass: those below a controller key `wanted` accepts, grouped by
/// that key, and the rest; both keep the order the findings were raised in.
pub(crate) fn group_by_controller(
    diagnostics: Diagnostics,
    wanted: impl Fn(&str) -> bool,
) -> (IndexMap<String, Diagnostics>, Diagnostics) {
    let mut by_key: IndexMap<String, Diagnostics> = IndexMap::new();
    let mut rest = Diagnostics::default();
    for finding in diagnostics {
        match controller_of(&finding.location.json_path).filter(|key| wanted(key)) {
            Some(key) => {
                if let Some(mine) = by_key.get_mut(key) {
                    mine.push(finding);
                } else {
                    let key = key.to_owned();
                    by_key.entry(key).or_default().push(finding);
                }
            }
            None => rest.push(finding),
        }
    }
    (by_key, rest)
}

/// Adds, beside each error whose text lies in a span an apostrophe kept from being
/// folded, an info naming the apostrophe's line.
fn folding_hints(diagnostics: &mut Diagnostics, folded: &Folded, original: &str) {
    let mut hints = Vec::new();
    for finding in diagnostics.iter() {
        let Some(span) = finding.location.span else {
            continue;
        };
        if finding.severity < Severity::Error {
            continue;
        }
        let Some(quote) = folded.quote_before(span.offset as usize) else {
            continue;
        };
        let offset = u32::try_from(quote).unwrap_or(u32::MAX);
        let (line, _) = Span { offset, len: 1 }.line_col(original);
        hints.push(Diagnostic::new(
            core_codes::CASE_FOLDING_DISABLED_AT_LINE.with([line.to_string()]),
            Severity::Info,
            Phase::Parse,
            finding.location.clone(),
        ));
    }
    for hint in hints {
        diagnostics.push(hint);
    }
}

// ---- the schema --------------------------------------------------------------------------------

/// A render controller file as its schema reads it. Both members are required; the required
/// check is the caller's ([`read_root`]), so that a missing `format_version` has its own code.
#[derive(Debug, PartialEq, Facet, Schema)]
struct Document {
    #[facet(schema::read_with = format_version)]
    format_version: Option<Written>,
    #[facet(schema::keys = CONTROLLERS)]
    render_controllers: Option<IndexMap<String, RenderController>>,
}

/// A `format_version` as written.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(opaque)]
struct Written(SemVersion);

bedrock_addon_core::impl_schema_leaf!(Written);
bedrock_addon_core::impl_no_molang!(Written);

/// The read of a whole file: the version and the controllers when the root's members validated.
fn read_root(
    text: &Text<'_>,
    read: &mut Read<'_>,
) -> Option<(SemVersion, IndexMap<String, RenderController>)> {
    let Document {
        format_version,
        render_controllers,
    } = read_document(&Node::parsed(text, JsonPath::new()), read)?;
    let root = Some(text.spans.root().span());
    if format_version.is_none() {
        let mut finding = Diagnostic::error(codes::REQUIRED_CHILD.with(["format_version"]));
        finding.location.span = root;
        read.push(finding);
    }
    if render_controllers.is_none() {
        let mut finding =
            Diagnostic::error(core_codes::REQUIRED_CHILD.with(["render_controllers"]));
        finding.location.span = root;
        read.push(finding);
    }
    Some((format_version?.0, render_controllers?))
}

/// The `format_version` node: a version string (`NoWildcards`; a partial `"1.10"` is 1.10.0)
/// within the schema's bounds.
fn format_version(node: &Node<'_>, read: &mut Read<'_>) -> Option<Written> {
    let written = node.value().as_string().map(facet_value::VString::as_str);
    let version = written
        .filter(|t| *t != "beta")
        .and_then(SemVersion::parse)
        .filter(|v| !v.is_any());
    let Some(version) = version else {
        read.error(node, codes::UNKNOWN_OPTION_TYPE.with(["SemVersion"]));
        return None;
    };
    let as_written = written.unwrap_or("");
    let refused = if version < V1_8_0 {
        Some(core_codes::VERSION_BELOW_MIN.with([as_written.to_owned(), V1_8_0.to_string()]))
    } else if version > MAXIMUM {
        Some(codes::ABOVE_MAX_VERSION.with([as_written.to_owned(), MAXIMUM.to_string()]))
    } else {
        None
    };
    if let Some(code) = refused {
        read.error(node, code);
        return None;
    }
    Some(Written(version))
}

/// The names that kept a capital, each at its path and spanning its value, in the order the
/// schema meets them: a controller key or an array name the game then registers under a name
/// no lower-cased reference reaches.
fn marked(
    controllers: &IndexMap<String, RenderController>,
    text: &Text<'_>,
) -> Vec<(JsonPath, Span)> {
    let capital = |name: &str| name.bytes().any(|b| b.is_ascii_uppercase());
    let mut out = Vec::new();
    let mut mark = |at: JsonPath| {
        if let Some(span) = text.spans.at(&at) {
            out.push((at, span));
        }
    };
    for (key, controller) in controllers {
        if capital(key) {
            mark(controller_path(key, None));
        }
        let Some(arrays) = &controller.arrays else {
            continue;
        };
        for (kind, lists) in [
            ("geometries", &arrays.geometries),
            ("materials", &arrays.materials),
            ("textures", &arrays.textures),
        ] {
            for name in lists.keys().filter(|name| capital(name)) {
                mark(controller_path(key, Some("arrays")).with(&[kind, name.as_str()]));
            }
        }
    }
    out
}

/// The `materials` values written as JSON numbers, by controller key and position in
/// [`RenderController::materials`] (the elements in order, inside one the patterns in sorted
/// order): the game reads them as the empty text when it parses the controller, after its
/// schema accepted the number.
fn numeric_materials(root: &Value) -> IndexSet<(String, usize)> {
    let mut out = IndexSet::new();
    let controllers = root
        .as_object()
        .and_then(|root| root.get("render_controllers"))
        .and_then(Value::as_object);
    for (key, controller) in controllers.into_iter().flat_map(|c| c.iter()) {
        let elements = controller
            .as_object()
            .and_then(|c| c.get("materials"))
            .and_then(Value::as_array);
        let mut values = Vec::new();
        for element in elements.into_iter().flatten() {
            let mut members: Vec<_> = element.as_object().into_iter().flatten().collect();
            members.sort_by(|a, b| a.0.as_str().as_bytes().cmp(b.0.as_str().as_bytes()));
            values.extend(members.into_iter().map(|(_, value)| value.is_number()));
        }
        for (index, _) in values.iter().enumerate().filter(|(_, numeric)| **numeric) {
            out.insert((key.as_str().to_owned(), index));
        }
    }
    out
}

/// An information, the crate's own, at each name that kept capitals, with the line of the
/// apostrophe that switched the folding off when there is one.
fn unresolvable_names(
    marked: Vec<(JsonPath, Span)>,
    folded: &Folded,
    original: &str,
    diagnostics: &mut Diagnostics,
) {
    for (at, span) in marked {
        let line = folded.quote_before(span.offset as usize).map(|quote| {
            let offset = u32::try_from(quote).unwrap_or(u32::MAX);
            Span { offset, len: 1 }.line_col(original).0
        });
        // The game says nothing, so this is an information.
        diagnostics.push(
            Diagnostic::of(match line {
                Some(line) => {
                    core_codes::CASE_FOLDING_DISABLED_AT_LINE_UNRESOLVABLE.with([line.to_string()])
                }
                None => Code::from(&core_codes::CASE_FOLDING_DISABLED_UNRESOLVABLE),
            })
            .at(at)
            .span(span),
        );
    }
}

/// A string node the controller parser reads as a Molang source, whose version pass 1 sets.
pub(crate) fn source(node: &Node<'_>, read: &mut Read<'_>) -> Option<MolangSource> {
    let text = String::from_json(node, read)?;
    Some(MolangSource::string(text, RawVersion(-1)))
}

/// "Required child  not found" with an empty name: an array whose required element option
/// found no element; the required-child line told as an invalid value, given whole.
fn no_element(node: &Node<'_>, read: &mut Read<'_>) {
    let line = core_codes::REQUIRED_CHILD.template().replacen("{}", "", 1);
    read.error(node, core_codes::INVALID_VALUE_WORDED.with([line]));
}

/// An array of at least one string, each a Molang source (`textures`, an `arrays` list).
pub(crate) fn sources(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<MolangSource>> {
    let list = read_each(node, read, source)?;
    if list.is_empty() {
        no_element(node, read);
        return None;
    }
    Some(list)
}

/// An `arrays` kind: an object of at least one `array.<name>` list.
pub(crate) fn named_lists(
    node: &Node<'_>,
    read: &mut Read<'_>,
) -> Option<IndexMap<String, Vec<MolangSource>>> {
    read_keyed(node, read, &ARRAYS, sources)
}

/// `materials` / `part_visibility`: an array of at least one object, each of at least one bone
/// pattern; the pairs in array order and, inside one object, in sorted key order.
fn bones<V>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    value: fn(&Node<'_>, &mut Read<'_>) -> Option<V>,
) -> Option<Vec<(BonePattern, V)>> {
    let elements = read_each(node, read, |element, read| {
        read_keyed(element, read, &BONES, value)
    })?;
    if elements.is_empty() {
        no_element(node, read);
        return None;
    }
    let pairs = elements.into_iter().flatten();
    Some(
        pairs
            .map(|(pattern, v)| (BonePattern(pattern), v))
            .collect(),
    )
}

/// `part_visibility`.
pub(crate) fn part_visibility(
    node: &Node<'_>,
    read: &mut Read<'_>,
) -> Option<Vec<(BonePattern, Molang)>> {
    bones(node, read, Molang::from_json)
}

/// `materials`: each value an expression the controller parser reads as a string with an empty
/// default: a string is its text, a bool `true` / `false`, a number the empty text.
pub(crate) fn materials(
    node: &Node<'_>,
    read: &mut Read<'_>,
) -> Option<Vec<(BonePattern, MolangSource)>> {
    let pairs = bones(node, read, material)?;
    let pairs = pairs.into_iter().map(|(pattern, value)| {
        let text = match value {
            Molang::Expr(source) => source.as_str().to_owned(),
            Molang::Bool(b) => b.to_string(),
            // A number (`Const`), and the variants the schema read gives none of.
            _ => String::new(),
        };
        (pattern, MolangSource::string(text, RawVersion(-1)))
    });
    Some(pairs.collect())
}

/// One `materials` value: an expression. The schema takes a number here and the controller
/// parser then reads it as the empty text, whatever the number: told at the member, since no
/// number is what the file is written with (`0` stands for the empty text).
fn material(node: &Node<'_>, read: &mut Read<'_>) -> Option<Molang> {
    let value = Molang::from_json(node, read)?;
    if matches!(value, Molang::Const(_)) {
        let mut finding = Diagnostic::of(&core_codes::INVALID_TYPE)
            .severity(Severity::Info)
            .at(node.path().clone());
        finding.location.span = node.span();
        read.push(finding);
    }
    Some(value)
}

/// An array of exactly two expressions (the node's minimum and maximum are both 2), told of by
/// its elements first, then by its length.
pub(super) fn pair(node: &Node<'_>, read: &mut Read<'_>) -> Option<[Molang; 2]> {
    let elements = read.array(node)?;
    let n = elements.len();
    let values: Vec<Molang> = elements
        .iter()
        .filter_map(|element| Molang::from_json(&element, read))
        .collect();
    let def = match n.cmp(&2) {
        std::cmp::Ordering::Less => Some(&core_codes::ARRAY_TOO_SMALL),
        std::cmp::Ordering::Greater => Some(&core_codes::ARRAY_TOO_LARGE),
        std::cmp::Ordering::Equal => None,
    };
    if let Some(def) = def {
        read.error(node, def.with([n.to_string(), "2".to_owned()]));
        return None;
    }
    values.try_into().ok()
}
