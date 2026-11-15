//! Geometry's schema read on the derived read: vanilla validates these files with JSON schema node
//! trees, leniently below 1.19.30 (an undeclared or wrong-typed member is removed and the file
//! loads) and strictly from 1.19.30 (it fails the file). This module holds the read of a file's
//! era, the legacy geometry-name pattern and the leaves only geometry reads.

use indexmap::IndexMap;

use crate::codes;
use crate::{CubeUv, Faces, Locator, Polys};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::kit::{Lossy, Node, Profile, Read, Recovery, Texts};
use bedrock_addon_core::json::parse::{Options, Parsed as Text, parse_slice};
use bedrock_addon_core::json::{JsonSlice, Value, Winner};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::schema::{
    self, FromJson, KeyPattern, Schema, UnknownRule, read_each, read_keyed,
};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::{MemberFailure, ParseCx, Reading};
use facet::Facet;

/// The crate's own text for a member below 1.19.30 that vanilla removes without a message: the
/// lenient read tells it with the member's key.
const CHILD_REMOVED: &str = "child '{}' not valid here (removed; format_version < 1.19.30)";

/// Below this version an undeclared or wrong-typed member is removed with an information; from
/// it, such a member fails the file.
pub(crate) const STRICT_FROM: FormatVersion = FormatVersion::new(1, 19, 30);

/// 1.8.0 and 1.12.0: the locator forms change there.
const V1_8_0: FormatVersion = known::V1_8_0;
const V1_12_0: FormatVersion = FormatVersion::new(1, 12, 0);
/// 1.19.30: an identifier may carry a namespace from here.
const V1_19_30: FormatVersion = STRICT_FROM;
/// 1.26.40: the first engine with the `shelf` transform.
const V1_26_40: FormatVersion = known::V1_26_40;

/// The read of a struct's payload under the rules of the game's geometry schemas: names as
/// written, the members in the file's order, lenient below 1.19.30 or strict, an object failed
/// only by an error raised below it, a lossy float told. The document's format gates the members
/// a later schema declares (`schema::since_format`) and picks the locator form and the
/// identifier rule.
pub(crate) fn read<'d>(cx: &'d ParseCx<'_>, findings: &'d mut Diagnostics) -> Read<'d> {
    let format = cx.original_version;
    let lenient = format < STRICT_FROM;
    let reading = Reading::TREE;
    let (reading, profile, texts) = if lenient {
        (
            reading.with_on_member_failure(MemberFailure::DefaultAndTell),
            Profile::TREE.with_recovery(Recovery::Value),
            Texts {
                unknown_named: None,
                unknown: Some(CHILD_REMOVED),
                ..Texts::TREE
            },
        )
    } else {
        (reading, Profile::TREE, Texts::TREE)
    };
    let read = Read::new(reading, findings)
        .profile(profile.with_lossy(Lossy::Told))
        .texts(texts)
        .load(cx.load)
        .versions(bedrock_addon_core::json::kit::Versions {
            format: Some(format),
            engine: None,
        });
    if lenient {
        read.unknown_severity(Severity::Info)
    } else {
        read
    }
}

/// Reads `payload` (the whole file) as `read` does, its findings into `d`: the value when no
/// error was raised. Of a member written twice the last one stands, whole and at its place.
pub(crate) fn read_payload<T>(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
    read_it: impl FnOnce(&Node<'_>, &mut Read<'_>) -> Option<T>,
) -> Option<T> {
    let text: Text<'_> = parse_slice(payload, Options::winner(Winner::Last)).ok()?;
    let mut found = Diagnostics::default();
    let mut reader = read(cx, &mut found);
    let value = read_it(&Node::parsed(&text, cx.at.clone()), &mut reader);
    drop(reader);
    let failed = found.has_errors();
    d.extend(found);
    value.filter(|_| !failed)
}

/// `geometry\.[a-zA-Z0-9_.'-:]+`, searched in the key (the legacy geometry members).
pub(crate) const GEOMETRY: KeyPattern = KeyPattern::Custom {
    matches: has_geometry_name,
    regex: r"geometry\.[a-zA-Z0-9_.'-:]+",
};

/// A `std::regex_search` for `geometry\.[a-zA-Z0-9_.'-:]+`: the text holds `geometry.` followed
/// by a letter, a digit, `_`, `.` or a character of the range `'` to `:` (which also admits
/// `( ) * + , - /`).
pub(crate) fn has_geometry_name(text: &str) -> bool {
    const PREFIX: &str = "geometry.";
    text.match_indices(PREFIX).any(|(at, _)| {
        text.as_bytes()
            .get(at + PREFIX.len())
            .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_' || (b'\''..=b':').contains(b))
    })
}

/// The document's format, which the read knows.
fn format(read: &Read<'_>) -> FormatVersion {
    read.known_versions().format.unwrap_or(known::V0_0_0)
}

/// An int node: a number without a fraction that fits an `i32`.
#[allow(
    clippy::cast_possible_truncation,
    reason = "the range is checked first"
)]
pub(crate) fn int(node: &Node<'_>, read: &mut Read<'_>) -> Option<i32> {
    let integral = node
        .value()
        .as_number()
        .map(facet_value::VNumber::to_f64_lossy)
        .filter(|x| x.fract() == 0.0 && (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(x));
    if integral.is_none() {
        read.wrong_accepting(node, "int");
    }
    integral.map(|x| x as i32)
}

/// An array of floats of any length (the locator vectors, which vanilla declares unsized): read
/// up to the first element that is no float.
pub(crate) fn float_list(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<f32>> {
    let elements = read.array(node)?;
    let mut values = Vec::with_capacity(elements.len());
    for element in elements.iter() {
        values.push(f32::from_json(&element, read)?);
    }
    Some(values)
}

/// Whether an array of `n` elements is the `want` the schema declares; if not, the size error
/// at the array.
fn sized(read: &mut Read<'_>, n: usize, want: usize, node: &Node<'_>) -> bool {
    if n == want {
        return true;
    }
    let def = if n < want {
        &core_codes::ARRAY_TOO_SMALL
    } else {
        &core_codes::ARRAY_TOO_LARGE
    };
    read.error(node, def.with([n.to_string(), want.to_string()]));
    false
}

/// An array of exactly `N` floats (`addChildArray(name, N, …)`): a wrong length is "Array too
/// small/large" and fails the file in either mode.
pub(crate) fn floats<const N: usize>(node: &Node<'_>, read: &mut Read<'_>) -> Option<[f32; N]> {
    let values = float_list(node, read)?;
    if !sized(read, values.len(), N, node) {
        return None;
    }
    values.try_into().ok()
}

/// A list of three-float arrays (`positions`, `normals`).
pub(crate) fn vectors3(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<[f32; 3]>> {
    read_each(node, read, floats::<3>)
}

/// A list of two-float arrays (`uvs`).
pub(crate) fn vectors2(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<[f32; 2]>> {
    read_each(node, read, floats::<2>)
}

/// A finding of `code` at `node` as the schema walk raised it, at `severity`.
fn told(node: &Node<'_>, code: Code, severity: Severity) -> Diagnostic {
    let mut finding = Diagnostic::error(code)
        .severity(severity)
        .at(node.path().clone());
    finding.location.span = node.span();
    finding
}

/// A box `uv`: two floats, of which vanilla keeps the whole part (`Json::Value::asInt`); a
/// fraction is told where it stands, an `InvalidValue` info as the clamp of the bounds is.
pub(crate) fn texels(node: &Node<'_>, read: &mut Read<'_>) -> Option<[f32; 2]> {
    let uv = floats::<2>(node, read)?;
    let elements = read.array(node)?;
    for (element, value) in elements.iter().zip(uv) {
        if value.fract() != 0.0 {
            let code = codes::UV_TRUNCATED.with([value.to_string(), value.trunc().to_string()]);
            read.push(told(&element, code, Severity::Info));
        }
    }
    Some(uv)
}

/// The three indices of a `poly_mesh` vertex: floats to the schema, read the way the game's
/// unsigned read takes them. An index that is not written as it was given is told at the
/// element with an `InvalidValue` information, whether a fraction goes (the game's reading) or
/// the value is below zero or beyond 32 bits (the bound is the crate's).
fn indices(node: &Node<'_>, read: &mut Read<'_>) -> Option<[u32; 3]> {
    // Read as `f64`, not as the floats of the schema: an index is not rounded to an `f32`, and
    // a number beyond `f32` is told once, by the index.
    let elements = read.array(node)?;
    let mut givens = Vec::with_capacity(elements.len());
    for element in elements.iter() {
        let Some(given) = element.value().as_number() else {
            read.wrong_accepting(&element, "float");
            return None;
        };
        givens.push(given.to_f64_lossy());
    }
    if !sized(read, givens.len(), 3, node) {
        return None;
    }
    let mut out = [0_u32; 3];
    for ((element, slot), given) in elements.iter().zip(out.iter_mut()).zip(givens) {
        // What truncates to an index of 32 bits; beyond it the cast's bound is the crate's own.
        let in_range = given > -1.0 && given < 4_294_967_296.0;
        let value = as_index(given);
        *slot = value;
        if !(in_range && given.fract() == 0.0) {
            // The crate's own information either way.
            let code = codes::INDEX_NOT_WHOLE.with([given.to_string(), value.to_string()]);
            read.push(told(&element, code, Severity::Info));
        }
    }
    Some(out)
}

/// `Json::Value::asUInt` of an index the schema declares as a float.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a float to integer cast saturates, as the index read does not need more"
)]
fn as_index(x: f64) -> u32 {
    x as u32
}

/// `render_group_id`: an int of at least 0 (the int node's inline minimum); a lower value fails
/// the file.
pub(crate) fn render_group_id(node: &Node<'_>, read: &mut Read<'_>) -> Option<u32> {
    let value = int(node, read)?;
    let Ok(id) = u32::try_from(value) else {
        let code = core_codes::OUTSIDE_RANGE.with([
            value.to_string(),
            "0".to_owned(),
            i32::MAX.to_string(),
        ]);
        read.error(node, code);
        return None;
    };
    Some(id)
}

/// `description.identifier` through the era's validator: `isValidGeometryIdentifier` (no `:`,
/// and the geometry name somewhere in the text) or, from 1.19.30,
/// `isValidOptionalNamespaceGeometryIdentifier` (the name after an optional `namespace:`). A
/// failing validator fails the file in either mode.
pub(crate) fn identifier(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    let identifier = String::from_json(node, read)?;
    let name = if format(read) >= V1_19_30 {
        identifier
            .split_once(':')
            .map_or(identifier.as_str(), |(_, name)| name)
    } else if identifier.contains(':') {
        read.error(node, codes::NO_NAMESPACE_CUSTOM.with([identifier]));
        return None;
    } else {
        identifier.as_str()
    };
    if !has_geometry_name(name) {
        read.error(node, Code::from(&core_codes::INVALID_VALUE));
        return None;
    }
    Some(identifier)
}

/// `uv_rotation` (`Geometry::_parseUvRotation`): 0, 90, 180 or 270 as an integer; anything else
/// is reported with vanilla's text, a kept error, and the face is not rotated (the member is
/// left out, failing nothing).
pub(crate) fn uv_rotation(node: &Node<'_>, read: &mut Read<'_>) -> Option<i32> {
    // An integer JSON value (`isInt` / `isUInt`) is compared as the number it is.
    let text = node.text().map_or("", |text| text.text);
    let quarter = node
        .value()
        .is_number()
        .then(|| text.parse::<i32>().ok())
        .flatten()
        .filter(|turn| matches!(turn, 0 | 90 | 180 | 270));
    if quarter.is_none() {
        // The game logs an error, uses rotation 0 and loads the geometry: a kept error.
        let finding = told(
            node,
            codes::INVALID_UV_ROTATION.with([text]),
            Severity::Error,
        );
        read.findings_mut().push_kept_error(finding);
    }
    quarter
}

/// `polys`: a string that holds `tri_list` or `quad_list`, or polygons of three or four
/// vertices.
pub(crate) fn polys(node: &Node<'_>, read: &mut Read<'_>) -> Option<Polys> {
    let value = node.value();
    if let Some(text) = value.as_string() {
        let text = text.as_str();
        // The valid values are regexes searched in the value.
        if !(text.contains("tri_list") || text.contains("quad_list")) {
            read.error(
                node,
                core_codes::NOT_ALLOWED_REGEX.with([text, "tri_list, quad_list"]),
            );
            return None;
        }
        return Some(Polys::List(text.to_owned()));
    }
    if !value.is_array() {
        read.wrong_accepting(node, "array, string");
        return None;
    }
    read_each(node, read, |poly, read| {
        let vertices = read_each(poly, read, indices)?;
        let n = vertices.len();
        let (def, bound) = match n {
            0..3 => (&core_codes::ARRAY_TOO_SMALL, "3"),
            3..=4 => return Some(vertices),
            _ => (&core_codes::ARRAY_TOO_LARGE, "4"),
        };
        read.error(poly, def.with([n.to_string(), bound.to_owned()]));
        None
    })
    .map(Polys::Indices)
}

/// A cube's `uv`: the box layout's two texels, or one rectangle per face.
impl FromJson for CubeUv {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let value = node.value();
        if value.is_array() {
            return texels(node, read).map(Self::Box);
        }
        if value.is_object() {
            return Faces::from_json(node, read).map(|faces| Self::Faces(Box::new(faces)));
        }
        read.wrong_accepting(node, "array, object");
        None
    }
}

/// `fit_to_frame`: declared under `gui` only; under another transform an undeclared member.
pub(crate) fn gui_only(node: &Node<'_>, read: &mut Read<'_>) -> Option<bool> {
    let mut parent = node.path().clone();
    parent.pop();
    if parent.last_key() != Some("gui") {
        read.unknown_by_rule(node, UnknownRule::Report);
        return None;
    }
    bool::from_json(node, read)
}

/// `item_display_transforms.shelf`: declared by an engine of 1.26.40 or later (in a file below
/// 1.26.40 the upgrade then rejects it); in an older game an undeclared member.
pub(crate) fn shelf(node: &Node<'_>, read: &mut Read<'_>) -> Option<crate::ItemDisplayTransform> {
    let engine = read
        .load_context()
        .map(|cx: &LoadContext| cx.engine_version);
    if engine.is_none_or(|engine| engine < V1_26_40) {
        read.unknown_by_rule(node, UnknownRule::Report);
        return None;
    }
    FromJson::from_json(node, read)
}

/// `minecraft:geometry`: the geometries, at least one, the minimum counted on the list as
/// written: a list whose every element the lenient read left out (below 1.19.30, each with its
/// information) held its elements when it was validated and loads, with no geometry.
pub(crate) fn geometries(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<crate::Geometry>> {
    let list = read_each(node, read, crate::Geometry::from_json)?;
    if node
        .value()
        .as_array()
        .is_some_and(facet_value::VArray::is_empty)
    {
        read.error(node, core_codes::ARRAY_TOO_SMALL.with(["0", "1"]));
        return None;
    }
    Some(list)
}

/// Which locator values a schema declares.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LocatorForm {
    /// Below 1.8.0: a vector, or an object whose `rotation` is optional.
    Pre18,
    /// 1.8.0 up to 1.12.0: a vector only.
    VectorOnly,
    /// From 1.12.0: a vector, or an object whose `rotation` is required.
    Modern,
}

/// A locator vector as read: three numbers, or an array of numbers of another length (told).
#[derive(Clone, Debug, PartialEq, Facet)]
#[repr(u8)]
enum Vector {
    /// Three numbers.
    Read([f32; 3]),
    /// An array of numbers of another length (reported).
    WrongLength,
}

/// A locator vector. Vanilla declares these arrays without a size; the crate reports a length
/// other than three and leaves the locator out.
impl FromJson for Vector {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let values = float_list(node, read)?;
        if let Ok(vector) = values.try_into() {
            return Some(Self::Read(vector));
        }
        let code = Code::from(&core_codes::INVALID_VALUE);
        read.push(told(node, code, Severity::Info));
        Some(Self::WrongLength)
    }
}

/// The object form of a locator below 1.8.0: `rotation` optional.
#[derive(Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang_with = no_molang)]
struct Placed {
    offset: Vector,
    rotation: Option<Vector>,
    ignore_inherited_scale: Option<bool>,
}

/// The object form of a locator from 1.12.0: `rotation` required.
#[derive(Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang_with = no_molang)]
struct PlacedModern {
    offset: Vector,
    rotation: Vector,
    ignore_inherited_scale: Option<bool>,
}

/// A locator form's walk: no Molang.
fn no_molang<T>(_: &mut T, _: &mut bedrock_addon_core::molang::MolangVisitor<'_>) {}

bedrock_addon_core::impl_schema_leaf!(Vector, CubeUv, Polys, Locator);
bedrock_addon_core::impl_no_molang!(Vector, CubeUv, Polys, Locator);

/// One locator: a vector, or (below 1.8.0 and from 1.12.0) the object of its offset, rotation and
/// scale rule. A vector of another length than three is told and the locator left out; in the
/// object form the locator goes with it, told dropped.
impl FromJson for Locator {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let format = format(read);
        let form = if format < V1_8_0 {
            LocatorForm::Pre18
        } else if format < V1_12_0 {
            LocatorForm::VectorOnly
        } else {
            LocatorForm::Modern
        };
        let value = node.value();
        if value.is_array() {
            return match Vector::from_json(node, read)? {
                Vector::Read(at) => Some(Self::Position(at)),
                Vector::WrongLength => None,
            };
        }
        let placed = match (value.is_object(), form) {
            (true, LocatorForm::Pre18) => Placed::from_json(node, read)?,
            (true, LocatorForm::Modern) => {
                let placed = PlacedModern::from_json(node, read)?;
                Placed {
                    offset: placed.offset,
                    rotation: Some(placed.rotation),
                    ignore_inherited_scale: placed.ignore_inherited_scale,
                }
            }
            (_, LocatorForm::VectorOnly) => {
                read.wrong_accepting(node, "array");
                return None;
            }
            _ => {
                read.wrong_accepting(node, "array, object");
                return None;
            }
        };
        let (Vector::Read(offset), rotation) = (placed.offset, placed.rotation) else {
            return dropped(node, read);
        };
        let rotation = match rotation {
            Some(Vector::Read(rotation)) => Some(rotation),
            Some(Vector::WrongLength) => return dropped(node, read),
            None => None,
        };
        Some(Self::Placed {
            offset,
            rotation,
            ignore_inherited_scale: placed.ignore_inherited_scale,
        })
    }
}

/// A placed locator left out for a vector of another length (the crate's own information): the
/// vector's finding does not say that the locator's other members go with it.
fn dropped(node: &Node<'_>, read: &mut Read<'_>) -> Option<Locator> {
    let code = Code::from(&core_codes::DROPPED_FIELD);
    read.push(told(node, code, Severity::Info));
    None
}

/// `locators`: names matching `[a-zA-Z0-9_.-]+`.
pub(crate) const LOCATORS: bedrock_addon_core::schema::Keys =
    bedrock_addon_core::schema::Keys::new(KeyPattern::IDENTIFIER);

/// The geometries of a file older than 1.12.0, `"geometry.<name>"` members of its root beside
/// `format_version` and `debug`.
enum RootMember<G> {
    /// `format_version`: read before the schema was chosen.
    Version,
    /// `debug`.
    Debug(bool),
    /// A `geometry.<name>` member.
    Geometry(G),
}

/// The root of a file older than 1.12.0: `format_version` (vanilla writes the version it
/// validates with into the file first, so the member is never missing), `debug` and at least
/// one `geometry.<name>` member.
pub(crate) fn read_root<G: FromJson>(
    node: &Node<'_>,
    read: &mut Read<'_>,
) -> Option<(Option<bool>, IndexMap<String, G>)> {
    /// The names the root takes: its two literal members, then the geometry pattern.
    fn named(key: &str) -> bool {
        matches!(key, "format_version" | "debug") || has_geometry_name(key)
    }
    const ROOT: bedrock_addon_core::schema::Keys =
        bedrock_addon_core::schema::Keys::new(KeyPattern::Custom {
            matches: named,
            regex: r"geometry\.[a-zA-Z0-9_.'-:]+",
        });
    let members = read_keyed(node, read, &ROOT, |member, read| {
        match member.path().last_key().unwrap_or_default() {
            "format_version" => Some(RootMember::Version),
            "debug" => bool::from_json(member, read).map(RootMember::Debug),
            _ => G::from_json(member, read).map(RootMember::Geometry),
        }
    })?;
    let mut debug = None;
    let mut geometries = IndexMap::new();
    for (key, member) in members {
        match member {
            RootMember::Version => {}
            RootMember::Debug(flag) => debug = Some(flag),
            RootMember::Geometry(geometry) => {
                geometries.insert(key, geometry);
            }
        }
    }
    if geometries.is_empty() {
        let object = read.object(node)?;
        read.missing_member(&object, GEOMETRY.text());
        return None;
    }
    Some((debug, geometries))
}

/// The first member `key` of the root object `root`.
pub(crate) fn root_member<'v>(root: &'v Value, key: &str) -> Option<&'v Value> {
    root.as_object()?.get(key)
}

/// The text a member of the root is written as.
pub(crate) fn written_text<'t>(text: &Text<'t>, key: &str) -> Option<&'t str> {
    let span = text.spans.at(&JsonPath::of_keys([key]))?;
    let source = text.spans.text();
    let start = (span.offset as usize).checked_sub(source.offset)?;
    source.text.get(start..start + span.len as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::diagnostic::JsonPath;
    use bedrock_addon_core::json::parse::parse;
    use bedrock_addon_core::schema::read_document;
    use bedrock_addon_core::versioning::ParseCx;

    #[test]
    fn the_geometry_pattern_is_searched_and_its_range_runs_from_quote_to_colon() {
        assert!(has_geometry_name("geometry.pig"));
        assert!(has_geometry_name("foo.geometry.bar"));
        assert!(has_geometry_name("geometry.a:geometry.b"));
        assert!(has_geometry_name("geometry.'"));
        assert!(has_geometry_name("geometry.:"));
        assert!(has_geometry_name("geometry.(x)"));
        assert!(!has_geometry_name("geometry."));
        assert!(!has_geometry_name("geometry.&"));
        assert!(!has_geometry_name("geometry.;"));
        assert!(!has_geometry_name("geometry pig"));
        assert!(!has_geometry_name("Geometry.pig"));
        let names = bedrock_addon_core::json::kit::Names::Exact;
        assert!(GEOMETRY.matches("x.geometry.pig", names));
    }

    /// Three members, one of them required.
    #[derive(Debug, PartialEq, Facet, Schema)]
    struct Three {
        a: Option<bool>,
        b: bool,
        x: Option<f32>,
    }

    /// `text` read as [`Three`] by the era of `version`: the value and the findings as
    /// `severity code at path`, in the order they are told.
    fn read_three(text: &str, version: FormatVersion) -> (Option<Three>, Vec<String>) {
        let parsed = parse(text, Options::default()).unwrap();
        let load = LoadContext::detached();
        let cx = ParseCx {
            load: &load,
            written: bedrock_addon_core::version::FormatVersionInput {
                version: Some(version),
                beta_literal: false,
                prerelease: false,
            },
            original_version: version,
            at: JsonPath::new(),
        };
        let mut found = Diagnostics::default();
        let mut reader = read(&cx, &mut found);
        let value = read_document::<Three>(&Node::parsed(&parsed, JsonPath::new()), &mut reader);
        drop(reader);
        let told = found
            .iter()
            .map(|d| {
                format!(
                    "{:?} {} at {}",
                    d.severity,
                    d.code.id(),
                    d.location.json_path
                )
            })
            .collect();
        (value, told)
    }

    /// W1: the members of an object are read in the file's order.
    #[test]
    fn members_are_read_in_document_order() {
        let (_, told) = read_three(r#"{"x": true, "b": 1, "a": 2}"#, STRICT_FROM);
        let paths: Vec<&str> = told.iter().map(|t| t.rsplit(' ').next().unwrap()).collect();
        assert_eq!(paths, ["x", "b", "a"]);
    }

    /// W2: a member whose read fails without an error fails nothing: the object is judged by
    /// the errors raised, and the required check is made.
    #[test]
    fn a_member_failed_without_an_error_fails_nothing_and_the_required_check_is_made() {
        let lenient = FormatVersion::new(1, 12, 0);
        let (value, told) = read_three(r#"{"a": 1, "b": true}"#, lenient);
        assert_eq!(
            value,
            Some(Three {
                a: None,
                b: true,
                x: None
            })
        );
        assert_eq!(told, ["Info core/unknown_child_option at a"]);
        let (value, told) = read_three(r#"{"a": 1}"#, lenient);
        assert_eq!(value, None);
        assert_eq!(
            told,
            [
                "Info core/unknown_child_option at a",
                "Error core/required_child at "
            ]
        );
    }

    /// W3: a number a float does not hold is read with a `LossyConversion` information.
    #[test]
    fn a_float_that_narrows_is_told_as_lossy() {
        let (value, told) = read_three(r#"{"b": true, "x": 0.123456789012}"#, STRICT_FROM);
        assert_eq!(value.and_then(|t| t.x), Some(0.123_456_79));
        assert_eq!(told, ["Info core/lossy_conversion at x"]);
    }
}
