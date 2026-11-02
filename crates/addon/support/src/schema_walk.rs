//! The walk the render-controller loader and the older fog structs run instead of a typed facet
//! parse: the game reads these files with schema trees whose member names are regexes matched
//! in full, which visit the members of an object in sorted key order and keep validating after
//! the first failure. Each object level is read from the file text as `IndexMap<String,
//! RawJson>` and each value at its own slice, so a failure in one entry does not stop the
//! others and findings carry file spans.

use std::sync::Arc;

use facet_json::RawJson;
use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, Span, VanillaText,
};
use bedrock_addon_core::json::{JsonSlice, Value, parse_at};
use bedrock_addon_core::molang::{
    JsonScalar, Molang, OtherJson, RawVersion, ReadField, ReaderKind,
};

// ---- vanilla's schema texts -----------------------------------------------------------------

/// A member no name group matches.
pub const CHILD_NOT_VALID: &str = "child '{}' not valid here.";
/// A matched name whose options all refuse the value's JSON type.
pub const UNKNOWN_OPTION_TYPE: &str =
    bedrock_addon_core::diagnostic::cereal::UNKNOWN_CHILD_OPTION_TEXT;
/// A required name group without a member.
pub const REQUIRED_CHILD: &str = bedrock_addon_core::diagnostic::cereal::REQUIRED_CHILD_TEXT;
/// A bool outside its allowed-value set.
pub const VALUE_NOT_ALLOWED: &str = "Value '{}' not allowed";
/// The string node's valid-value regexes.
pub const NOT_ALLOWED_REGEX: &str =
    "Value '{}' not allowed.  Must match at least one of the following regex patterns: {}";
/// The float node's range.
pub const OUTSIDE_RANGE: &str = "Value '{}' outside valid range [{}, {}]";
/// The array node's minimum element count.
pub const ARRAY_TOO_SMALL: &str = "Array too small ({} < {})";
/// The array node's maximum element count.
pub const ARRAY_TOO_LARGE: &str = "Array too large ({} > {})";
/// A version below the node's minimum.
pub const BELOW_MIN_VERSION: &str = "Version {} below min required version ({})";
/// A version above the node's maximum.
pub const ABOVE_MAX_VERSION: &str = "Version {} greater than max allowed version ({})";

/// A vanilla text with its arguments, as the code fields take it.
#[allow(
    clippy::unnecessary_wraps,
    reason = "every code field is an `Option<VanillaText>`"
)]
#[must_use]
pub fn text(template: &'static str, args: &[&str]) -> Option<VanillaText> {
    Some(VanillaText::Formatted {
        template,
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
    })
}

/// C's `%f` of a float: six decimals, no exponent.
#[must_use]
pub fn percent_f(x: f32) -> String {
    format!("{:.6}", f64::from(x))
}

// ---- JSON nodes --------------------------------------------------------------------------------

/// The JSON type of a node, from its first byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    /// `null`, or no text at all.
    Null,
    /// `true` or `false`.
    Bool,
    /// A number.
    Number,
    /// A string.
    String,
    /// An array.
    Array,
    /// An object.
    Object,
}

/// One JSON value of the file, as its slice of the text (trimmed to the value's own tokens).
#[derive(Clone, Copy, Debug)]
pub struct Node<'a> {
    slice: JsonSlice<'a>,
}

impl<'a> Node<'a> {
    /// The value of a slice (comments and whitespace around it skipped); `None` when the text
    /// is not JSON.
    #[must_use]
    pub fn of(slice: JsonSlice<'a>) -> Option<Self> {
        let raw: RawJson<'a> = parse_at(slice).ok()?;
        slice.nested(&raw).map(|slice| Self { slice })
    }

    /// The JSON type of the node, from its first byte.
    #[must_use]
    pub fn kind(&self) -> Kind {
        match self.slice.text.as_bytes().first() {
            Some(b'{') => Kind::Object,
            Some(b'[') => Kind::Array,
            Some(b'"') => Kind::String,
            Some(b't' | b'f') => Kind::Bool,
            Some(b'n') | None => Kind::Null,
            Some(_) => Kind::Number,
        }
    }

    /// The value's text.
    #[must_use]
    pub fn text(&self) -> &'a str {
        self.slice.text
    }

    /// The node's place in the file, saturated to `u32`.
    #[must_use]
    pub fn span(&self) -> Span {
        let to_u32 = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
        Span {
            offset: to_u32(self.slice.offset),
            len: to_u32(self.slice.text.len()),
        }
    }

    /// The value as an untyped JSON value (an unknown member kept for write-back).
    #[must_use]
    pub fn value(&self) -> Option<Value> {
        parse_at::<Value>(self.slice).ok()
    }

    fn nested(&self, raw: &RawJson<'a>) -> Option<Node<'a>> {
        self.slice.nested(raw).map(|slice| Node { slice })
    }
}

// ---- member-name patterns ---------------------------------------------------------------------

/// A member name or pattern of a schema object, matched in full and as written.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Pattern {
    /// A literal member name.
    Name(&'static str),
    /// `<prefix>[a-zA-Z0-9_.:-]+` with an escaped dotted prefix (`controller\.render\.`); the
    /// second text is the regex as vanilla prints it.
    Prefixed(&'static str, &'static str),
    /// `[a-zA-Z0-9_.:*]+`: a bone pattern.
    Bone,
}

impl Pattern {
    /// Whether `key` is a name this pattern accepts.
    #[must_use]
    pub fn matches(self, key: &str) -> bool {
        match self {
            Self::Name(name) => key == name,
            Self::Prefixed(prefix, _) => key
                .strip_prefix(prefix)
                .is_some_and(|tail| charset(tail, b"_.:-")),
            Self::Bone => charset(key, b"_.:*"),
        }
    }

    /// The pattern as vanilla prints it in "Required child %s not found".
    #[must_use]
    pub fn text(self) -> &'static str {
        match self {
            Self::Name(name) => name,
            Self::Prefixed(_, regex) => regex,
            Self::Bone => "[a-zA-Z0-9_.:*]+",
        }
    }
}

/// `[a-zA-Z0-9<extra>]+`, in full.
#[must_use]
pub fn charset(key: &str, extra: &[u8]) -> bool {
    !key.is_empty()
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || extra.contains(&b))
}

/// One name group of a schema object: its pattern and whether a member must match it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Group {
    /// The pattern a member name is matched against.
    pub pattern: Pattern,
    /// A member must match the pattern.
    pub required: bool,
}

impl Group {
    /// An optional group of one fixed name.
    #[must_use]
    pub const fn name(name: &'static str) -> Self {
        Self {
            pattern: Pattern::Name(name),
            required: false,
        }
    }

    /// A required group of one fixed name.
    #[must_use]
    pub const fn required(name: &'static str) -> Self {
        Self {
            pattern: Pattern::Name(name),
            required: true,
        }
    }

    /// A group of any pattern.
    #[must_use]
    pub const fn pattern(pattern: Pattern, required: bool) -> Self {
        Self { pattern, required }
    }
}

// ---- the walk ---------------------------------------------------------------------------------

/// The walk's state: the findings so far and the breadcrumb of the value being read.
#[derive(Debug, Default)]
pub struct Walk {
    /// The findings of the walk so far.
    pub diagnostics: Diagnostics,
    path: JsonPath,
    /// The members a reader marked for a later check, by path and span.
    pub marked: Vec<(JsonPath, Span)>,
}

impl Walk {
    /// A walk whose findings are reported below `at`.
    #[must_use]
    pub fn with_path(at: &JsonPath) -> Self {
        Self {
            diagnostics: Diagnostics::default(),
            path: at.clone(),
            marked: Vec::new(),
        }
    }

    /// Marks the member at the current path.
    pub fn mark(&mut self, span: Span) {
        self.marked.push((self.path.clone(), span));
    }

    /// Descend into the member `key`.
    pub fn enter(&mut self, key: &str) {
        self.path.push(key);
    }

    /// Descend into the element `index`.
    pub fn enter_index(&mut self, index: usize) {
        self.path.push_index(index);
    }

    /// Go back up one level.
    pub fn leave(&mut self) {
        self.path.pop();
    }

    /// A finding at the current path.
    pub fn push(&mut self, code: DiagnosticCode, severity: Severity, span: Option<Span>) {
        self.diagnostics.push(Diagnostic::new(
            code,
            severity,
            Phase::Parse,
            Location::new(None, None, self.path.clone(), span),
        ));
    }

    /// A kept error at the current path: the game logs an error line and goes on loading
    /// the document.
    pub fn push_kept(&mut self, code: DiagnosticCode, span: Option<Span>) {
        self.diagnostics.push_kept_error(Diagnostic::new(
            code,
            Severity::Error,
            Phase::Parse,
            Location::new(None, None, self.path.clone(), span),
        ));
    }

    /// A schema error at the current path.
    pub fn error(&mut self, code: DiagnosticCode, span: Span) {
        self.push(code, Severity::Error, Some(span));
    }

    /// "unknown child schema option type.  Allowed types: …" at the current path.
    pub fn bad_type(&mut self, allowed: &str, node: Node<'_>) {
        self.error(
            bedrock_addon_core::diagnostic::cereal::unknown_child_option(allowed),
            node.span(),
        );
    }

    /// "child '<key>' not valid here." at the member.
    pub fn unknown(&mut self, key: &str, node: Node<'_>) {
        self.enter(key);
        self.error(
            DiagnosticCode::UnrecognizedField {
                text: text(CHILD_NOT_VALID, &[key]),
            },
            node.span(),
        );
        self.leave();
    }

    /// "Required child <name> not found" at the current path.
    pub fn missing(&mut self, name: &str, node: Node<'_>) {
        self.error(
            DiagnosticCode::MissingRequiredField {
                text: text(REQUIRED_CHILD, &[name]),
            },
            node.span(),
        );
    }

    /// An `InvalidValue` error with one of vanilla's texts at the current path.
    pub fn invalid(&mut self, template: &'static str, args: &[&str], node: Node<'_>) {
        self.error(
            DiagnosticCode::InvalidValue {
                text: text(template, args),
            },
            node.span(),
        );
    }

    /// The members of an object node in document order (of a key written twice the last value
    /// stands, as jsoncpp keeps it); `None` when the node is not an object.
    #[allow(clippy::unused_self, reason = "read like the other node accessors")]
    #[must_use]
    pub fn members<'a>(&self, node: Node<'a>) -> Option<Vec<(String, Node<'a>)>> {
        if node.kind() != Kind::Object {
            return None;
        }
        let map: IndexMap<String, RawJson<'a>> = parse_at(node.slice).ok()?;
        map.iter()
            .map(|(key, raw)| Some((key.clone(), node.nested(raw)?)))
            .collect()
    }

    /// The elements of an array node; `None` when the node is not an array.
    #[allow(clippy::unused_self, reason = "read like the other node accessors")]
    #[must_use]
    pub fn elements<'a>(&self, node: Node<'a>) -> Option<Vec<Node<'a>>> {
        if node.kind() != Kind::Array {
            return None;
        }
        let raw: Vec<RawJson<'a>> = parse_at(node.slice).ok()?;
        raw.iter().map(|raw| node.nested(raw)).collect()
    }

    /// Walks an object node against its name groups, the members in sorted key order (the
    /// order of jsoncpp's `std::map`): each member is handed to `on` with the index of the
    /// first group it matches; an unmatched member is "child '%s' not valid here.". When every
    /// member was read, each required group without a member is "Required child %s not found".
    /// `false` when anything failed.
    pub fn object<'a>(
        &mut self,
        node: Node<'a>,
        groups: &[Group],
        mut on: impl FnMut(&mut Self, usize, &str, Node<'a>) -> bool,
    ) -> bool {
        let Some(mut members) = self.members(node) else {
            self.bad_type("object", node);
            return false;
        };
        members.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        let mut ok = true;
        let mut found = vec![false; groups.len()];
        for (key, value) in members {
            let Some(index) = groups.iter().position(|g| g.pattern.matches(&key)) else {
                self.unknown(&key, value);
                ok = false;
                continue;
            };
            self.enter(&key);
            let read = on(self, index, &key, value);
            self.leave();
            if read {
                found[index] = true;
            } else {
                ok = false;
            }
        }
        if ok {
            for (group, found) in groups.iter().zip(found) {
                if group.required && !found {
                    self.missing(group.pattern.text(), node);
                    ok = false;
                }
            }
        }
        ok
    }

    /// Walks an array node: each element handed to `on` at its index. `false` when the node is
    /// not an array or an element failed.
    pub fn array<'a>(
        &mut self,
        node: Node<'a>,
        mut on: impl FnMut(&mut Self, usize, Node<'a>) -> bool,
    ) -> bool {
        let Some(elements) = self.elements(node) else {
            self.bad_type("array", node);
            return false;
        };
        let mut ok = true;
        for (index, element) in elements.into_iter().enumerate() {
            self.enter_index(index);
            ok &= on(self, index, element);
            self.leave();
        }
        ok
    }

    // ---- typed leaves ----

    /// A string node, decoded.
    pub fn string(&mut self, node: Node<'_>) -> Option<String> {
        if node.kind() != Kind::String {
            self.bad_type("string", node);
            return None;
        }
        parse_at::<String>(node.slice).ok()
    }

    /// A number node.
    #[allow(
        clippy::cast_possible_truncation,
        reason = "vanilla reads these members as float"
    )]
    pub fn float(&mut self, node: Node<'_>) -> Option<f32> {
        if node.kind() != Kind::Number {
            self.bad_type("float", node);
            return None;
        }
        let number = parse_at::<f64>(node.slice).ok()?;
        let nearest = number as f32;
        if crate::lossy::narrows(number) {
            // The game holds the member as a float, and the document is written with the
            // float: digits of the file a float does not hold are noted by the crate's own
            // information.
            self.push(
                DiagnosticCode::LossyConversion,
                Severity::Info,
                Some(node.span()),
            );
        }
        if nearest.is_infinite() {
            // A number beyond a float is the largest finite float of its sign, with the
            // information of such a number on its token; the file loads, and the written text reads
            // back to the same value without a finding.
            self.push(
                DiagnosticCode::InvalidValue { text: None },
                Severity::Info,
                Some(node.span()),
            );
            return Some(f32::MAX.copysign(nearest));
        }
        Some(nearest)
    }

    /// A bool node.
    pub fn bool(&mut self, node: Node<'_>) -> Option<bool> {
        match (node.kind(), node.slice.text) {
            (Kind::Bool, "true") => Some(true),
            (Kind::Bool, "false") => Some(false),
            _ => {
                self.bad_type("bool", node);
                None
            }
        }
    }

    /// An `ExpressionNode` member: a number, bool or string (`ReaderKind::SchemaValidated`);
    /// anything else is the type error. The context version is set by pass 1.
    pub fn molang(&mut self, node: Node<'_>) -> Option<Molang> {
        let scalar = match node.kind() {
            Kind::Number => JsonScalar::Number(parse_at::<f64>(node.slice).ok()?),
            Kind::Bool => JsonScalar::Bool(node.slice.text == "true"),
            Kind::String => JsonScalar::String(Arc::from(parse_at::<String>(node.slice).ok()?)),
            Kind::Null => JsonScalar::Other(OtherJson::Null),
            Kind::Array => JsonScalar::Other(OtherJson::Array),
            Kind::Object => JsonScalar::Other(OtherJson::Object),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixed_names_match_in_full_and_as_written() {
        let p = Pattern::Prefixed(
            "controller.render.",
            r"controller\.render\.[a-zA-Z0-9_.:-]+",
        );
        assert!(p.matches("controller.render.a:b-c_d.E9"));
        assert!(!p.matches("controller.render."));
        assert!(!p.matches("Controller.render.a"));
        assert!(!p.matches("controller.render.a b"));
        assert!(!p.matches("controller.render.a*"));
    }

    #[test]
    fn bone_patterns_take_a_star_and_no_hyphen() {
        assert!(Pattern::Bone.matches("arm*_L.0:x"));
        assert!(!Pattern::Bone.matches("left-arm"));
        assert!(!Pattern::Bone.matches(""));
        assert!(!Pattern::Bone.matches("a b"));
    }

    #[test]
    fn percent_f_prints_six_decimals_without_an_exponent() {
        assert_eq!(percent_f(-5.0), "-5.000000");
        assert_eq!(
            percent_f(f32::MAX),
            "340282346638528859811704183484516925440.000000"
        );
    }
}
