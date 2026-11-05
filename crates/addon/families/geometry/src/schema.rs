//! The walk the geometry loader runs instead of a typed facet parse: vanilla validates these
//! files with JSON schema node trees, leniently below 1.19.30 (an undeclared or
//! wrong-typed member is removed and the file loads) and strictly from 1.19.30 (it fails the
//! file). Each object level is read from the file text as `IndexMap<String, RawJson>` and each
//! value at its own slice, so that one bad entry does not hide the others and findings carry
//! file spans.
//!
//! In lenient mode the game collects the names the schema does not declare and removes them
//! from the object; in strict mode each one is an error.

use std::sync::Arc;

use facet_json::RawJson;
use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity, Span, VanillaText,
};
use bedrock_addon_core::json::{JsonSlice, parse_at};
use bedrock_addon_core::molang::{
    JsonScalar, Molang, OtherJson, RawVersion, ReadField, ReaderKind,
};

// ---- vanilla's texts -----------------------------------------------------------------------------

/// The text for a member the schema does not declare (strict validation).
pub(crate) const CHILD_NOT_VALID: &str = "child '{}' not valid here.";
/// The crate's own text for the same member below 1.19.30, where vanilla removes it without a
/// message.
pub(crate) const CHILD_REMOVED: &str =
    "child '{}' not valid here (removed; format_version < 1.19.30)";
/// The text for a declared name whose options all refuse the value's JSON type.
pub(crate) const UNKNOWN_OPTION_TYPE: &str =
    bedrock_addon_core::diagnostic::cereal::UNKNOWN_CHILD_OPTION_TEXT;
/// The text for a required member that is absent.
pub(crate) const REQUIRED_CHILD: &str = bedrock_addon_core::diagnostic::cereal::REQUIRED_CHILD_TEXT;
/// The array node's minimum element count.
pub(crate) const ARRAY_TOO_SMALL: &str = "Array too small ({} < {})";
/// The array node's maximum element count.
pub(crate) const ARRAY_TOO_LARGE: &str = "Array too large ({} > {})";
/// The int node's range (`render_group_id`).
pub(crate) const OUTSIDE_RANGE: &str = "Value '{}' outside valid range [{}, {}]";
/// The string node's valid-value regexes (`poly_mesh.polys`).
pub(crate) const NOT_ALLOWED_REGEX: &str =
    "Value '{}' not allowed.  Must match at least one of the following regex patterns: {}";
/// The version node's maximum (the 1.8 schema's 1.11.0).
pub(crate) const ABOVE_MAX_VERSION: &str = "Version {} greater than max allowed version ({})";
/// The text for a file whose schema validation failed.
pub(crate) const DID_NOT_VALIDATE: &str = "geometry file didn't validate";
/// The text for a file the JSON reader could not parse.
pub(crate) const PARSE_ERROR: &str = "parse error:\n{}";
/// The text for a root that is not an object.
pub(crate) const NOT_AN_OBJECT: &str = "geometry file could not be loaded: expected the contents to be inside a JSON object - The file has to start with an \"{\" and end with a \"}\"";
/// The text for a file newer than the game.
pub(crate) const EXCEEDS_GAME_VERSION: &str =
    "Error: geometry file format version: {} exceeds game version: {}. Skipping.";
/// The text for a base-game `models/blocks/` file below 1.19.30.
pub(crate) const INTERNAL_BLOCKS_FLOOR: &str =
    "Internal blocks are required to use geometry version >= 1.19.30";
/// The text for a namespace in a geometry identifier outside the base game.
pub(crate) const NO_NAMESPACE_CUSTOM: &str =
    "Error: namespaces are not supported for custom geometry: identifier: {}";
/// The identifier loop: a namespace in a base-game entity geometry.
pub(crate) const NO_NAMESPACE_INTERNAL: &str =
    "Error: namespaces are not supported for internal entity geometry: identifier: {}";
/// The identifier loop: a base-game block geometry outside `minecraft:`.
pub(crate) const NEEDS_MINECRAFT_NAMESPACE: &str =
    "Error: Internal block geometries required the minecraft namespace: {}";
/// The text for `embedded` below 1.21.110.
pub(crate) const EMBEDDED_TOO_OLD: &str = "\"item_display_transform\" for \"{}\" is invalid - \"embedded\" can only be used in a file versioned 1.21.110 or higher";
/// The text for `shelf` below 1.26.40.
pub(crate) const SHELF_TOO_OLD: &str = "\"item_display_transform\" for \"{}\" is invalid - \"{}\" can only be used in a file version 1.26.40 or higher";
/// The text for a display-transform vector outside its range.
pub(crate) const TRANSFORM_OUT_OF_BOUNDS: &str = "\"item_display_transform\" - {} ({}, {}, {}) is out of bounds. Allowed range: ({}, {}, {}) to ({}, {}, {}).";
/// The text for a face rotation that is not a quarter turn.
pub(crate) const INVALID_UV_ROTATION: &str =
    "Invalid uv rotation value '{}', valid values are 0, 90, 180 and 270";
/// The text for an index-less mesh whose lists differ in length.
pub(crate) const NO_POLYS_SECTION: &str = "No 'polys' section, and the number of positions, normals, and UVs aren't equal ({}, {}, {} resp)";
/// The text for a `tri_list` whose positions are not triangles.
pub(crate) const NOT_TRIANGLES: &str =
    "\"polys\" specifies \"tri_list\" but positions[] isn't a multiple of three?";
/// The text for a `quad_list` whose positions are not quads.
pub(crate) const NOT_QUADS: &str =
    "\"polys\" specifies \"quad_list\" but positions[] isn't a multiple of four?";
/// The model build: a bone that names itself as its parent.
pub(crate) const PARENT_IS_SELF: &str = "Error: parenting model part to itself!";
/// The crate's own text for an unreadable `format_version`.
pub(crate) const VERSION_NOT_VALID: &str =
    "format_version '{}' is not a valid version; treated as 1.0.0";
/// The crate's own text for a legacy parent no pack defines.
pub(crate) const PARENT_NOT_DEFINED: &str =
    "geometry '{}' inherits from '{}', which is not defined";
/// The crate's own text for a bone whose parent is no bone of the geometry.
pub(crate) const BONE_PARENT_MISSING: &str =
    "bone '{}' has parent '{}', which is not a bone of geometry '{}'";
/// The crate's own text for a clamped bound.
pub(crate) const BOUNDS_CLAMPED: &str = "visible_bounds_{} {} clamped to {}";
/// The crate's own text for a box `uv` component with a fraction, which the game's integer
/// read drops.
pub(crate) const UV_TRUNCATED: &str = "box uv {} is read as the whole number {}";
/// The crate's own text for a bone's `name` or `parent` written in another case than the lower
/// case it is read in.
pub(crate) const BONE_LOWER_CASED: &str = "bone {} '{}' is read as '{}'";
/// The crate's own text for a `poly_mesh` index that is no whole number of 32 bits, which the
/// game's unsigned read turns into one.
pub(crate) const INDEX_NOT_WHOLE: &str = "poly_mesh index {} is read as {}";
/// The crate's own text for the geometry-level `cape`, which is lost.
pub(crate) const CAPE_DROPPED: &str = "cape is not carried over from format 1.8 geometry";

/// A vanilla text with its arguments, as the code fields take it.
#[allow(
    clippy::unnecessary_wraps,
    reason = "every code field is an `Option<VanillaText>`"
)]
pub(crate) fn text(template: &'static str, args: &[&str]) -> Option<VanillaText> {
    Some(VanillaText::Formatted {
        template,
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
    })
}

// ---- JSON nodes --------------------------------------------------------------------------------

/// The JSON type of a node, from its first byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JsonKind {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

/// One JSON value of the file, as its slice of the text (trimmed to the value's own tokens).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Node<'a> {
    slice: JsonSlice<'a>,
}

impl<'a> Node<'a> {
    /// The value of a slice (comments and whitespace around it skipped); `None` when the text
    /// is not JSON.
    pub(crate) fn of(slice: JsonSlice<'a>) -> Option<Self> {
        let raw: RawJson<'a> = parse_at(slice).ok()?;
        slice.nested(&raw).map(|slice| Self { slice })
    }

    pub(crate) fn kind(&self) -> JsonKind {
        match self.slice.text.as_bytes().first() {
            Some(b'{') => JsonKind::Object,
            Some(b'[') => JsonKind::Array,
            Some(b'"') => JsonKind::String,
            Some(b't' | b'f') => JsonKind::Bool,
            Some(b'n') | None => JsonKind::Null,
            Some(_) => JsonKind::Number,
        }
    }

    /// The value's text.
    pub(crate) fn text(&self) -> &'a str {
        self.slice.text
    }

    pub(crate) fn span(&self) -> Span {
        let to_u32 = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
        Span {
            offset: to_u32(self.slice.offset),
            len: to_u32(self.slice.text.len()),
        }
    }

    fn nested(&self, raw: &RawJson<'a>) -> Option<Node<'a>> {
        self.slice.nested(raw).map(|slice| Node { slice })
    }
}

// ---- member names --------------------------------------------------------------------------------

/// A member name or pattern of a schema object.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Pattern {
    /// A literal member name.
    Name(&'static str),
    /// `geometry\.[a-zA-Z0-9_.'-:]+`, searched in the key (the legacy geometry members).
    Geometry,
    /// `[a-zA-Z0-9_.-]+` in full (locator names).
    Identifier,
}

impl Pattern {
    pub(crate) fn matches(self, key: &str) -> bool {
        match self {
            Self::Name(name) => key == name,
            Self::Geometry => has_geometry_name(key),
            Self::Identifier => {
                !key.is_empty()
                    && key
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
            }
        }
    }

    /// The pattern as vanilla prints it in "Required child %s not found".
    pub(crate) fn text(self) -> &'static str {
        match self {
            Self::Name(name) => name,
            Self::Geometry => r"geometry\.[a-zA-Z0-9_.'-:]+",
            Self::Identifier => "[a-zA-Z0-9_.-]+",
        }
    }
}

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

/// One name group of a schema object: its pattern and whether a member must match it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Group {
    pub(crate) pattern: Pattern,
    pub(crate) required: bool,
}

impl Group {
    pub(crate) const fn name(name: &'static str) -> Self {
        Self {
            pattern: Pattern::Name(name),
            required: false,
        }
    }

    pub(crate) const fn required(name: &'static str) -> Self {
        Self {
            pattern: Pattern::Name(name),
            required: true,
        }
    }

    pub(crate) const fn pattern(pattern: Pattern, required: bool) -> Self {
        Self { pattern, required }
    }
}

/// What reading one member gave.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Hit {
    /// The member was read.
    Ok,
    /// The member was not read: its finding is reported (an error, or in lenient mode the
    /// warning of a removed member).
    Failed,
}

impl<T> From<Option<T>> for Hit {
    fn from(value: Option<T>) -> Self {
        if value.is_some() {
            Self::Ok
        } else {
            Self::Failed
        }
    }
}

// ---- the walk ------------------------------------------------------------------------------------

/// The walk's state: the findings so far, the breadcrumb of the value being read, and the
/// validation mode.
#[derive(Debug)]
pub(crate) struct SchemaWalk {
    pub(crate) diagnostics: Diagnostics,
    path: JsonPath,
    /// Below 1.19.30: an undeclared or wrong-typed member is a warning and is left out.
    lenient: bool,
    /// How many errors the walk raised (a member that fails without one was removed leniently).
    errors: usize,
}

impl SchemaWalk {
    pub(crate) fn new(lenient: bool, at: &JsonPath) -> Self {
        Self {
            diagnostics: Diagnostics::default(),
            path: at.clone(),
            lenient,
            errors: 0,
        }
    }

    pub(crate) fn enter(&mut self, key: &str) {
        self.path.push(key);
    }

    pub(crate) fn enter_index(&mut self, index: usize) {
        self.path.push_index(index);
    }

    pub(crate) fn leave(&mut self) {
        self.path.pop();
    }

    /// A finding at the current path.
    pub(crate) fn push(&mut self, code: DiagnosticCode, severity: Severity, span: Span) {
        if severity == Severity::Error {
            self.errors += 1;
        }
        self.diagnostics.push(
            Diagnostic::error(code)
                .severity(severity)
                .at(self.path.clone())
                .span(span),
        );
    }

    /// A content-log error of the game's after which the geometry loads: an error-level finding
    /// after which the game keeps the document (a kept error); it fails no validation.
    pub(crate) fn push_kept(&mut self, code: DiagnosticCode, span: Span) {
        self.diagnostics
            .push_kept_error(Diagnostic::error(code).at(self.path.clone()).span(span));
    }

    /// A schema error at the current path: the file does not validate in either mode.
    pub(crate) fn error(&mut self, code: DiagnosticCode, span: Span) {
        self.push(code, Severity::Error, span);
    }

    /// "unknown child schema option type.  Allowed types: …" at the current path: an error
    /// under strict validation, a warning and the member's removal under lenient validation.
    pub(crate) fn bad_type(&mut self, allowed: &str, node: Node<'_>) {
        // Lenient: the member is removed without the game's log, so the crate reports an
        // information.
        let severity = if self.lenient {
            Severity::Info
        } else {
            Severity::Error
        };
        self.push(
            DiagnosticCode::InvalidType {
                text: text(UNKNOWN_OPTION_TYPE, &[allowed]),
            },
            severity,
            node.span(),
        );
    }

    /// An undeclared member: "child '<key>' not valid here." and the file fails, or below
    /// 1.19.30 an information (vanilla removes it without a message; the information is the
    /// crate's own) and the member is left out. The schemas from 1.12 on declare no `reset`,
    /// `neverRender` or `binding`: a `reset` of such a file is removed as vanilla removes it.
    fn unknown(&mut self, key: &str, node: Node<'_>) {
        self.enter(key);
        if self.lenient {
            self.push(
                DiagnosticCode::UnrecognizedField {
                    text: text(CHILD_REMOVED, &[key]),
                },
                Severity::Info,
                node.span(),
            );
        } else {
            self.error(
                DiagnosticCode::UnrecognizedField {
                    text: text(CHILD_NOT_VALID, &[key]),
                },
                node.span(),
            );
        }
        self.leave();
    }

    /// "Required child <pattern> not found" at the current path.
    fn missing(&mut self, pattern: Pattern, node: Node<'_>) {
        self.error(
            DiagnosticCode::MissingRequiredField {
                text: text(REQUIRED_CHILD, &[pattern.text()]),
            },
            node.span(),
        );
    }

    /// The members of an object node, in file order. The loader hands the walk a text in which
    /// no object repeats a key (`json::last_wins`).
    #[allow(clippy::unused_self, reason = "read like the other node accessors")]
    pub(crate) fn members<'a>(&self, node: Node<'a>) -> Option<Vec<(String, Node<'a>)>> {
        if node.kind() != JsonKind::Object {
            return None;
        }
        let map: IndexMap<String, RawJson<'a>> = parse_at(node.slice).ok()?;
        map.iter()
            .map(|(key, raw)| Some((key.clone(), node.nested(raw)?)))
            .collect()
    }

    /// The elements of an array node.
    #[allow(clippy::unused_self, reason = "read like the other node accessors")]
    pub(crate) fn elements<'a>(&self, node: Node<'a>) -> Option<Vec<Node<'a>>> {
        if node.kind() != JsonKind::Array {
            return None;
        }
        let raw: Vec<RawJson<'a>> = parse_at(node.slice).ok()?;
        raw.iter().map(|raw| node.nested(raw)).collect()
    }

    /// Walks an object node against its name groups: each member is handed to `on` with the index
    /// of the first group it matches; an unmatched member is undeclared. When every member was
    /// read, each required group without a member is "Required child %s not found". `false`
    /// when the node does not validate.
    pub(crate) fn object<'a>(
        &mut self,
        node: Node<'a>,
        groups: &[Group],
        mut on: impl FnMut(&mut Self, usize, &str, Node<'a>) -> Hit,
    ) -> bool {
        let Some(members) = self.members(node) else {
            self.bad_type("object", node);
            return false;
        };
        let before = self.errors;
        let mut found = vec![false; groups.len()];
        for (key, value) in members {
            let Some(index) = groups.iter().position(|g| g.pattern.matches(&key)) else {
                self.unknown(&key, value);
                continue;
            };
            self.enter(&key);
            if on(self, index, &key, value) == Hit::Ok {
                found[index] = true;
            }
            self.leave();
        }
        if self.errors == before {
            for (group, found) in groups.iter().zip(found) {
                if group.required && !found {
                    self.missing(group.pattern, node);
                }
            }
        }
        self.errors == before
    }

    /// Walks an array node: each element handed to `on` at its index; an element `on` refuses
    /// without an error was removed leniently. `false` when the node does not validate.
    pub(crate) fn array<'a>(
        &mut self,
        node: Node<'a>,
        mut on: impl FnMut(&mut Self, usize, Node<'a>) -> Hit,
    ) -> bool {
        let Some(elements) = self.elements(node) else {
            self.bad_type("array", node);
            return false;
        };
        let before = self.errors;
        for (index, element) in elements.into_iter().enumerate() {
            self.enter_index(index);
            on(self, index, element);
            self.leave();
        }
        self.errors == before
    }

    /// A list of values read by `read` (`bones`, `cubes`, `texture_meshes`).
    pub(crate) fn list<'a, T>(
        &mut self,
        node: Node<'a>,
        mut read: impl FnMut(&mut Self, Node<'a>) -> Option<T>,
    ) -> Option<Vec<T>> {
        let mut out = Vec::new();
        let ok = self.array(node, |walk, _, element| {
            read(walk, element).map(|value| out.push(value)).into()
        });
        ok.then_some(out)
    }

    // ---- typed leaves ----

    /// A string node, decoded.
    pub(crate) fn string(&mut self, node: Node<'_>) -> Option<String> {
        if node.kind() != JsonKind::String {
            self.bad_type("string", node);
            return None;
        }
        parse_at::<String>(node.slice).ok()
    }

    /// A number node.
    pub(crate) fn number(&mut self, node: Node<'_>) -> Option<f64> {
        if node.kind() != JsonKind::Number {
            self.bad_type("float", node);
            return None;
        }
        parse_at::<f64>(node.slice).ok()
    }

    /// A number node as `f32`.
    #[allow(
        clippy::cast_possible_truncation,
        reason = "vanilla reads these members as float"
    )]
    pub(crate) fn float(&mut self, node: Node<'_>) -> Option<f32> {
        let number = self.number(node)?;
        if bedrock_addon_support::lossy::narrows(number) {
            // The game holds the member as a float, and the geometry is written with the
            // float: digits of the file a float does not hold are noted (the crate's own information).
            self.push(DiagnosticCode::LossyConversion, Severity::Info, node.span());
        }
        Some(self.finite(number, node) as f32)
    }

    /// `number` as far as an `f32` holds it. A number whose nearest `f32` is infinite is the
    /// largest finite float of its sign, with an information; the game's float read gives the
    /// infinity, which no writer can spell, so the written file would not read back.
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the cast is what decides whether the number fits"
    )]
    fn finite(&mut self, number: f64, node: Node<'_>) -> f64 {
        if (number as f32).is_finite() {
            return number;
        }
        // The crate's own information: the game says nothing here.
        self.push(
            DiagnosticCode::InvalidValue { text: None },
            Severity::Info,
            node.span(),
        );
        f64::from(f32::MAX).copysign(number)
    }

    /// An int node: a number without a fraction that fits an `i32`.
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the range is checked first"
    )]
    pub(crate) fn int(&mut self, node: Node<'_>) -> Option<i32> {
        let integral = (node.kind() == JsonKind::Number)
            .then(|| parse_at::<f64>(node.slice).ok())
            .flatten()
            .filter(|x| {
                x.fract() == 0.0 && (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(x)
            });
        if integral.is_none() {
            self.bad_type("int", node);
        }
        integral.map(|x| x as i32)
    }

    /// A bool node.
    pub(crate) fn bool(&mut self, node: Node<'_>) -> Option<bool> {
        match (node.kind(), node.slice.text) {
            (JsonKind::Bool, "true") => Some(true),
            (JsonKind::Bool, "false") => Some(false),
            _ => {
                self.bad_type("bool", node);
                None
            }
        }
    }

    /// An array of exactly `N` floats (`addChildArray(name, N, …)`): a wrong length is "Array
    /// too small/large" and fails the file in either mode.
    pub(crate) fn floats<const N: usize>(&mut self, node: Node<'_>) -> Option<[f32; N]> {
        let values = self.float_list(node)?;
        if !self.sized(values.len(), N, node) {
            return None;
        }
        values.try_into().ok()
    }

    /// Whether an array of `n` elements is the `want` the schema declares; if not, the size error
    /// at the array.
    fn sized(&mut self, n: usize, want: usize, node: Node<'_>) -> bool {
        if n == want {
            return true;
        }
        let template = if n < want {
            ARRAY_TOO_SMALL
        } else {
            ARRAY_TOO_LARGE
        };
        self.error(
            DiagnosticCode::InvalidValue {
                text: text(template, &[&n.to_string(), &want.to_string()]),
            },
            node.span(),
        );
        false
    }

    /// A box `uv`: two floats, of which vanilla keeps the whole part (`Json::Value::asInt`,
    /// fraction is told where it stands, an `InvalidValue` info as the clamp of the bounds is.
    pub(crate) fn texels(&mut self, node: Node<'_>) -> Option<[f32; 2]> {
        let uv = self.floats::<2>(node)?;
        for (index, (element, value)) in self
            .elements(node)
            .unwrap_or_default()
            .into_iter()
            .zip(uv)
            .enumerate()
        {
            if value.fract() != 0.0 {
                self.enter_index(index);
                self.push(
                    DiagnosticCode::InvalidValue {
                        text: text(
                            UV_TRUNCATED,
                            &[&value.to_string(), &value.trunc().to_string()],
                        ),
                    },
                    Severity::Info,
                    element.span(),
                );
                self.leave();
            }
        }
        Some(uv)
    }

    /// The three indices of a `poly_mesh` vertex: floats to the schema, read
    /// the way the game's unsigned read takes them. An index that is not written as it was given is
    /// told at the element with an `InvalidValue` information, whether a fraction goes (the
    /// game's reading) or the value is below zero or beyond 32 bits (the bound is the crate's).
    pub(crate) fn indices(&mut self, node: Node<'_>) -> Option<[u32; 3]> {
        // Read as `f64`, not as the floats of the schema: an index is not rounded to an `f32`,
        // and a number beyond `f32` is told once, by the index.
        let Some(elements) = self.elements(node) else {
            self.bad_type("array", node);
            return None;
        };
        let mut read = [0_u32; 3];
        let mut givens = Vec::with_capacity(elements.len());
        for (index, element) in elements.iter().enumerate() {
            self.enter_index(index);
            let given = self.number(*element);
            self.leave();
            givens.push(given?);
        }
        if !self.sized(givens.len(), 3, node) {
            return None;
        }
        for (index, ((element, slot), given)) in elements
            .into_iter()
            .zip(read.iter_mut())
            .zip(givens)
            .enumerate()
        {
            // What truncates to an index of 32 bits; beyond it the cast's bound is the crate's own.
            let in_range = given > -1.0 && given < 4_294_967_296.0;
            let value = as_index(given);
            *slot = value;
            if in_range && given.fract() == 0.0 {
                continue;
            }
            // The crate's own information either way.
            let severity = Severity::Info;
            let _ = in_range;
            self.enter_index(index);
            self.push(
                DiagnosticCode::InvalidValue {
                    text: text(INDEX_NOT_WHOLE, &[&given.to_string(), &value.to_string()]),
                },
                severity,
                element.span(),
            );
            self.leave();
        }
        Some(read)
    }

    /// An array of floats of any length (the locator vectors, which vanilla declares unsized).
    pub(crate) fn float_list(&mut self, node: Node<'_>) -> Option<Vec<f32>> {
        let Some(elements) = self.elements(node) else {
            self.bad_type("array", node);
            return None;
        };
        let mut values = Vec::with_capacity(elements.len());
        for (index, element) in elements.into_iter().enumerate() {
            self.enter_index(index);
            let value = self.float(element);
            self.leave();
            values.push(value?);
        }
        Some(values)
    }

    /// An `ExpressionNode` member: a number, bool or string (`ReaderKind::SchemaValidated`);
    /// anything else is the type error. The context version is set by pass 1.
    pub(crate) fn molang(&mut self, node: Node<'_>) -> Option<Molang> {
        let scalar = match node.kind() {
            JsonKind::Number => JsonScalar::Number(parse_at::<f64>(node.slice).ok()?),
            JsonKind::Bool => JsonScalar::Bool(node.slice.text == "true"),
            JsonKind::String => JsonScalar::String(Arc::from(parse_at::<String>(node.slice).ok()?)),
            JsonKind::Null => JsonScalar::Other(OtherJson::Null),
            JsonKind::Array => JsonScalar::Other(OtherJson::Array),
            JsonKind::Object => JsonScalar::Other(OtherJson::Object),
        };
        // A number beyond `f32` is the largest finite float, with core's one warning at the
        // number.
        let read = bedrock_addon_core::molang::read_finite(
            scalar,
            ReaderKind::SchemaValidated,
            RawVersion(-1),
        );
        if let Ok((ReadField::Value(value), clamped)) = read {
            if clamped {
                self.diagnostics
                    .push(bedrock_addon_core::molang::beyond_f32_warning(
                        self.path.clone(),
                        Some(node.span()),
                    ));
            }
            Some(value)
        } else {
            self.bad_type("ExpressionNode", node);
            None
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn a_locator_name_matches_in_full() {
        assert!(Pattern::Identifier.matches("lead.hold-1_a"));
        assert!(!Pattern::Identifier.matches("a b"));
        assert!(!Pattern::Identifier.matches(""));
    }
}
