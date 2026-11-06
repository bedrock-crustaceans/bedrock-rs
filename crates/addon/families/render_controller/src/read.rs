//! Reading render controller files: the case fold, the one strict schema walked over the text
//! (file stage), then the per-controller stage, the Molang passes and the crate's own findings.

use std::collections::HashMap;

use facet_value::VObject;
use indexmap::{IndexMap, IndexSet};

use super::fold::{Folded, fold};
use super::molang::{MolangRun, run};
use super::{
    BonePattern, RenderController, RenderControllerArrays, RenderControllerFile, Rgba, UvAnim,
};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, JsonPathSegment, Phase, Severity, Span,
    VanillaText,
};
use bedrock_addon_core::json::node_tree::{
    ABOVE_MAX_VERSION, ARRAY_TOO_LARGE, ARRAY_TOO_SMALL, BELOW_MIN_VERSION, Failure, Group, Hit,
    Kind, Lossy, Names, Node, Order, Pattern, REQUIRED_CHILD, Rules, UNKNOWN_OPTION_TYPE,
    VALUE_NOT_ALLOWED, Validation, Walk, text,
};
use bedrock_addon_core::json::{
    JsonSlice, MAX_JSON_DEPTH, Value, check_depth, last_wins, parse_at,
};
use bedrock_addon_core::json::{PrepareError, Prepared};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::{Molang, MolangSource, RawVersion};
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::version::{FormatVersionInput, SemVersion};
use bedrock_addon_shared_shapes::TrueOnly;

/// The schema's `format_version` minimum and the family's floor.
pub(crate) const V1_8_0: SemVersion = SemVersion::new(1, 8, 0);
/// The version node's default maximum, which the schema never overrides.
pub(crate) const MAXIMUM: SemVersion = SemVersion::new(65535, 0, 0);

/// `error parsing %s:\n%s`: the file's path and the JSON reader's messages.
const ERROR_PARSING: &str = "error parsing {}:\n{}";
/// The text for a controller without a `textures` member.
const MUST_CONTAIN_TEXTURES: &str = "Render controller must contain textures.";

/// The rules of the game's render controller schema: names as written, the members of an
/// object in sorted key order (jsoncpp's `std::map`), strict, an object failed by any member,
/// and a lossy float told.
pub(crate) const RULES: Rules = Rules {
    names: Names::Exact,
    order: Order::Sorted,
    validation: Validation::Strict,
    failure: Failure::MemberFailed,
    lossy: Lossy::Told,
};

/// The controller key pattern.
const CONTROLLER_KEY: Pattern = Pattern::Prefixed {
    prefix: "controller.render.",
    extra: b"_.:-",
    regex: r"controller\.render\.[a-zA-Z0-9_.:-]+",
};
/// The array name pattern of every `arrays` kind.
const ARRAY_KEY: Pattern = Pattern::Prefixed {
    prefix: "array.",
    extra: b"_.:-",
    regex: r"array\.[a-zA-Z0-9_.:-]+",
};
/// `[a-zA-Z0-9_.:*]+`: a bone pattern.
const BONE: Pattern = Pattern::Charset {
    extra: b"_.:*",
    regex: "[a-zA-Z0-9_.:*]+",
};

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

/// The controllers as the schema walk reads them, with what the later stages need to know and
/// the model does not keep.
#[derive(Default)]
struct Drafts {
    controllers: IndexMap<String, RenderController>,
    /// The controllers without a `textures` member.
    without_textures: Vec<String>,
    /// `materials` values written as JSON numbers, by controller key and position in
    /// [`RenderController::materials`]: the game reads them as the empty text when it parses
    /// the controller, after its schema accepted the number.
    numeric_materials: IndexSet<(String, usize)>,
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
        && let Ok(written) = bedrock_addon_core::json::from_slice_jsonc::<Value>(bytes)
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
        diagnostics.push(Diagnostic::error(DiagnosticCode::MalformedInput {
            text: text(ERROR_PARSING, &[path.as_str(), &message]),
            message,
        }));
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
    // The whole text is parsed first: a syntax error anywhere drops the file.
    if let Err(e) = parse_at::<Value>(JsonSlice::new(&folded.text)) {
        return malformed(e.error.to_string());
    }
    // Of a key written twice (also two keys the fold made equal) the last value stands.
    let cleaned = last_wins(&folded.text);
    let Some(root) = Node::of(JsonSlice::new(&cleaned)) else {
        return malformed("not a JSON value".to_owned());
    };

    let mut walk = Walk::new(RULES, &JsonPath::new());
    let mut drafts = Drafts::default();
    let written = read_root(&mut walk, root, &mut drafts);
    let marked = std::mem::take(&mut walk.marked);
    let mut diagnostics = walk.diagnostics;
    let Some(written) = written.filter(|_| !diagnostics.has_errors()) else {
        folding_hints(&mut diagnostics, &folded, original);
        return Parsed::failed(diagnostics, None);
    };

    let at_version = || JsonPath::of_keys(["format_version"]);
    unresolvable_names(marked, &folded, original, &mut diagnostics);
    if written.is_pre_release() {
        diagnostics.push(
            Diagnostic::info(DiagnosticCode::PrereleaseFormatVersion {
                version: written.to_string(),
            })
            .at(at_version()),
        );
    }
    if written.format_version() > cx.engine_version {
        // The game compares the version with nothing but the schema's bounds.
        diagnostics.push(
            Diagnostic::info(DiagnosticCode::FormatVersionAboveEngine {
                version: written.to_string(),
            })
            .at(at_version()),
        );
    }

    let mut file = RenderControllerFile {
        format_version: written.format_version(),
        render_controllers: drafts.controllers,
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
            numeric_materials: &drafts.numeric_materials,
        },
        &mut diagnostics,
    );
    if outcome.file_rejected {
        return Parsed::failed(diagnostics, Some(written));
    }

    // The controller stage: `RenderController::parse` returned an error for these.
    let mut failed: IndexSet<String> = outcome.dropped;
    for key in drafts.without_textures {
        diagnostics.push(
            Diagnostic::error(DiagnosticCode::MissingRequiredField {
                text: Some(VanillaText::Fixed(MUST_CONTAIN_TEXTURES)),
            })
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
            DiagnosticCode::CaseFoldingDisabled {
                line: Some(line),
                unresolvable: false,
            },
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

/// The walk of a whole file; the version when the root's members validated.
fn read_root(walk: &mut Walk, root: Node<'_>, drafts: &mut Drafts) -> Option<SemVersion> {
    if root.kind() != Kind::Object {
        walk.bad_type("object", root);
        return None;
    }
    let mut version = None;
    let mut controllers = false;
    // Both members are required; the required check is written out below so that a missing
    // `format_version` has its own code.
    let groups = [
        Group::name("format_version"),
        Group::name("render_controllers"),
    ];
    let ok = walk.object(root, &groups, |walk, group, _, node| {
        if group == 0 {
            // A member that is present counts as found even when its value fails.
            version = Some(format_version(walk, node));
            matches!(version, Some(Some(_))).into()
        } else {
            controllers = true;
            read_controllers(walk, node, drafts).into()
        }
    });
    if !ok {
        return None;
    }
    if version.is_none() {
        walk.error(
            DiagnosticCode::MissingFormatVersion {
                text: text(REQUIRED_CHILD, &["format_version"]),
            },
            root.span(),
        );
    }
    if !controllers {
        walk.missing("render_controllers", root);
    }
    version.flatten().filter(|_| controllers)
}

/// The `format_version` node: a version string (`NoWildcards`; a partial `"1.10"` is 1.10.0)
/// within the schema's bounds.
fn format_version(walk: &mut Walk, node: Node<'_>) -> Option<SemVersion> {
    let written = if node.kind() == Kind::String {
        parse_at::<String>(JsonSlice::new(node.text())).ok()
    } else {
        None
    };
    let version = written
        .as_deref()
        .filter(|t| *t != "beta")
        .and_then(SemVersion::parse)
        .filter(|v| !v.is_any());
    let Some(version) = version else {
        walk.error(
            DiagnosticCode::UnsupportedVersion {
                text: text(UNKNOWN_OPTION_TYPE, &["SemVersion"]),
            },
            node.span(),
        );
        return None;
    };
    let as_written = written.as_deref().unwrap_or("");
    let bound = if version < V1_8_0 {
        Some((BELOW_MIN_VERSION, V1_8_0))
    } else if version > MAXIMUM {
        Some((ABOVE_MAX_VERSION, MAXIMUM))
    } else {
        None
    };
    if let Some((template, bound)) = bound {
        walk.error(
            DiagnosticCode::UnsupportedVersion {
                text: text(template, &[as_written, &bound.to_string()]),
            },
            node.span(),
        );
        return None;
    }
    Some(version)
}

/// Marks the member `name` at the walk's path when it kept a capital: a controller key or an
/// array name the game then registers under a name no lower-cased reference reaches.
fn mark_capitals(walk: &mut Walk, name: &str, value: Node<'_>) {
    if name.bytes().any(|b| b.is_ascii_uppercase()) {
        walk.mark(value.span());
    }
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
            Diagnostic::info(DiagnosticCode::CaseFoldingDisabled {
                line,
                unresolvable: true,
            })
            .at(at)
            .span(span),
        );
    }
}

fn read_controllers(walk: &mut Walk, node: Node<'_>, drafts: &mut Drafts) -> bool {
    if node.kind() != Kind::Object {
        walk.bad_type("object", node);
        return false;
    }
    walk.object(
        node,
        &[Group::pattern(CONTROLLER_KEY, true)],
        |walk, _, key, v| {
            if v.kind() != Kind::Object {
                walk.bad_type("object", v);
                return Hit::Failed;
            }
            mark_capitals(walk, key, v);
            read_controller(walk, v, key, drafts).into()
        },
    )
}

/// The members of a controller, in the schema's order (the order its required check reports).
const CONTROLLER_MEMBERS: [Group; 14] = [
    Group::name("rebuild_animation_matrices"),
    Group::name("arrays"),
    Group::required("geometry"),
    Group::name("part_visibility"),
    Group::name("materials"),
    Group::name("textures"),
    Group::name("color"),
    Group::name("overlay_color"),
    Group::name("on_fire_color"),
    Group::name("is_hurt_color"),
    Group::name("uv_anim"),
    Group::name("light_color_multiplier"),
    Group::name("ignore_lighting"),
    Group::name("filter_lighting"),
];

/// A source whose version pass 1 sets.
fn source(text: String) -> MolangSource {
    MolangSource::string(text, RawVersion(-1))
}

#[allow(clippy::too_many_lines, reason = "one arm per member of the schema")]
fn read_controller(walk: &mut Walk, node: Node<'_>, key: &str, drafts: &mut Drafts) -> bool {
    let mut geometry = None;
    let mut textures = None;
    let mut numeric = Vec::new();
    let mut c = RenderController {
        geometry: source(String::new()),
        textures: Vec::new(),
        materials: Vec::new(),
        part_visibility: Vec::new(),
        arrays: None,
        color: None,
        overlay_color: None,
        on_fire_color: None,
        is_hurt_color: None,
        uv_anim: None,
        light_color_multiplier: None,
        ignore_lighting: None,
        filter_lighting: None,
        rebuild_animation_matrices: None,
    };
    let ok = walk.object(node, &CONTROLLER_MEMBERS, |walk, group, _, v| match group {
        0 => walk
            .bool(v)
            .map(|x| c.rebuild_animation_matrices = Some(x))
            .into(),
        1 => read_arrays(walk, v).map(|x| c.arrays = Some(x)).into(),
        2 => walk.string(v).map(|x| geometry = Some(x)).into(),
        3 => read_bones(walk, v, Walk::molang)
            .map(|x| c.part_visibility = x)
            .into(),
        4 => read_bones(walk, v, |walk, v| {
            let value = walk.molang(v)?;
            // The schema takes a number here and the controller parser then
            // reads it as the empty text, whatever the number: told at the member,
            // since no number is what the file is written with (`0` stands for the empty
            // text).
            if matches!(value, Molang::Const(_)) {
                walk.push(
                    DiagnosticCode::InvalidType { text: None },
                    Severity::Info,
                    Some(v.span()),
                );
            }
            Some(value)
        })
        .map(|entries| {
            for (index, (pattern, value)) in entries.into_iter().enumerate() {
                // The element is read as a string with an empty default: a string is its text, a
                // bool `true` / `false`, a number the empty text.
                let text = match value {
                    Molang::Expr(source) => source.as_str().to_owned(),
                    Molang::Bool(b) => b.to_string(),
                    Molang::Const(_) => {
                        numeric.push(index);
                        String::new()
                    }
                    // The enum is non-exhaustive; the schema reader gives the three above.
                    _ => String::new(),
                };
                c.materials.push((pattern, source(text)));
            }
        })
        .into(),
        5 => read_strings(walk, v).map(|x| textures = Some(x)).into(),
        6 => read_color(walk, v).map(|x| c.color = Some(x)).into(),
        7 => read_color(walk, v)
            .map(|x| c.overlay_color = Some(x))
            .into(),
        8 => read_color(walk, v)
            .map(|x| c.on_fire_color = Some(x))
            .into(),
        9 => read_color(walk, v)
            .map(|x| c.is_hurt_color = Some(x))
            .into(),
        10 => read_uv_anim(walk, v).map(|x| c.uv_anim = Some(x)).into(),
        11 => walk
            .molang(v)
            .map(|x| c.light_color_multiplier = Some(x))
            .into(),
        12 => walk.bool(v).map(|x| c.ignore_lighting = Some(x)).into(),
        _ => match walk.bool(v) {
            Some(true) => {
                c.filter_lighting = Some(TrueOnly);
                Hit::Ok
            }
            Some(false) => {
                // The node's allowed-value set is `{true}`.
                walk.invalid(VALUE_NOT_ALLOWED, &["false"], v);
                Hit::Failed
            }
            None => Hit::Failed,
        },
    });
    let (true, Some(geometry)) = (ok, geometry) else {
        return false;
    };
    c.geometry = source(geometry);
    match textures {
        Some(textures) => c.textures = textures,
        None => drafts.without_textures.push(key.to_owned()),
    }
    drafts
        .numeric_materials
        .extend(numeric.into_iter().map(|index| (key.to_owned(), index)));
    drafts.controllers.insert(key.to_owned(), c);
    true
}

/// "Required child  not found" with an empty name: an array whose required element option
/// found no element.
fn no_element(walk: &mut Walk, node: Node<'_>) {
    walk.invalid(REQUIRED_CHILD, &[""], node);
}

/// An array of at least one string, each a Molang source.
fn read_strings(walk: &mut Walk, node: Node<'_>) -> Option<Vec<MolangSource>> {
    let mut list = Vec::new();
    let ok = walk.array(node, |walk, _, element| {
        walk.string(element).map(|s| list.push(source(s))).into()
    });
    if ok && list.is_empty() {
        no_element(walk, node);
        return None;
    }
    ok.then_some(list)
}

/// `arrays`: the three kinds, each an object of at least one `array.<name>` list.
fn read_arrays(walk: &mut Walk, node: Node<'_>) -> Option<RenderControllerArrays> {
    if node.kind() != Kind::Object {
        walk.bad_type("object", node);
        return None;
    }
    let mut arrays = RenderControllerArrays::default();
    let groups = [
        Group::name("geometries"),
        Group::name("materials"),
        Group::name("textures"),
    ];
    let ok = walk.object(node, &groups, |walk, group, _, v| {
        if v.kind() != Kind::Object {
            walk.bad_type("object", v);
            return Hit::Failed;
        }
        let mut map = IndexMap::new();
        let ok = walk.object(v, &[Group::pattern(ARRAY_KEY, true)], |walk, _, name, v| {
            mark_capitals(walk, name, v);
            read_strings(walk, v)
                .map(|list| map.insert(name.to_owned(), list))
                .into()
        });
        match group {
            0 => arrays.geometries = map,
            1 => arrays.materials = map,
            _ => arrays.textures = map,
        }
        ok.into()
    });
    ok.then_some(arrays)
}

/// `materials` / `part_visibility`: an array of at least one object, each of at least one bone
/// pattern; the pairs in array order and, inside one object, in sorted key order.
fn read_bones(
    walk: &mut Walk,
    node: Node<'_>,
    mut value: impl FnMut(&mut Walk, Node<'_>) -> Option<Molang>,
) -> Option<Vec<(BonePattern, Molang)>> {
    let mut list = Vec::new();
    let mut elements = 0_usize;
    let ok = walk.array(node, |walk, _, element| {
        elements += 1;
        if element.kind() != Kind::Object {
            walk.bad_type("object", element);
            return Hit::Failed;
        }
        walk.object(
            element,
            &[Group::pattern(BONE, true)],
            |walk, _, pattern, v| {
                value(walk, v)
                    .map(|m| list.push((BonePattern(pattern.to_owned()), m)))
                    .into()
            },
        )
        .into()
    });
    if ok && elements == 0 {
        no_element(walk, node);
        return None;
    }
    ok.then_some(list)
}

fn read_color(walk: &mut Walk, node: Node<'_>) -> Option<Rgba> {
    if node.kind() != Kind::Object {
        walk.bad_type("object", node);
        return None;
    }
    let mut color = Rgba::default();
    let groups = [
        Group::name("r"),
        Group::name("g"),
        Group::name("b"),
        Group::name("a"),
    ];
    let ok = walk.object(node, &groups, |walk, group, _, v| {
        let Some(value) = walk.molang(v) else {
            return Hit::Failed;
        };
        match group {
            0 => color.r = Some(value),
            1 => color.g = Some(value),
            2 => color.b = Some(value),
            _ => color.a = Some(value),
        }
        Hit::Ok
    });
    ok.then_some(color)
}

fn read_uv_anim(walk: &mut Walk, node: Node<'_>) -> Option<UvAnim> {
    if node.kind() != Kind::Object {
        walk.bad_type("object", node);
        return None;
    }
    let mut offset = None;
    let mut scale = None;
    let groups = [Group::required("offset"), Group::required("scale")];
    let ok = walk.object(node, &groups, |walk, group, _, v| {
        let pair = read_pair(walk, v);
        let read = pair.is_some();
        if group == 0 {
            offset = pair;
        } else {
            scale = pair;
        }
        read.into()
    });
    match (ok, offset, scale) {
        (true, Some(offset), Some(scale)) => Some(UvAnim { offset, scale }),
        _ => None,
    }
}

/// An array of exactly two expressions (the node's minimum and maximum are both 2).
pub(super) fn read_pair(walk: &mut Walk, node: Node<'_>) -> Option<[Molang; 2]> {
    // The array is read once, for its elements and for its length.
    let elements = walk.elements(node);
    if elements.is_none() {
        walk.bad_type("array", node);
    }
    if node.kind() != Kind::Array {
        return None;
    }
    let elements = elements.unwrap_or_default();
    let n = elements.len();
    let mut values = Vec::new();
    for (index, element) in elements.into_iter().enumerate() {
        walk.enter_index(index);
        values.extend(walk.molang(element));
        walk.leave();
    }
    let template = match n.cmp(&2) {
        std::cmp::Ordering::Less => Some(ARRAY_TOO_SMALL),
        std::cmp::Ordering::Greater => Some(ARRAY_TOO_LARGE),
        std::cmp::Ordering::Equal => None,
    };
    if let Some(template) = template {
        walk.invalid(template, &[&n.to_string(), "2"], node);
        return None;
    }
    values.try_into().ok()
}
