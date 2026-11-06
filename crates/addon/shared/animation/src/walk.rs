//! The walk both animation loaders run instead of a typed facet parse: the game reads these files
//! with schema trees whose member names match case-insensitively and in full, whose pattern
//! members (`animation\.…`, time keys) are regexes, and which keep validating after the first
//! failure. Each object level is read from the file text as `IndexMap<String, RawJson>` and
//! each value at its own slice, so a failure in one entry does not stop the others and
//! findings carry file spans.

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

// ---- vanilla's schema texts -------------------------------------------------------------------

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
/// The enum node's valid keys.
pub const NOT_ALLOWED_KEYS: &str =
    "Value '{}' not allowed.  Must match at least one of the following keys: {}";
/// The array node's minimum element count.
pub const ARRAY_TOO_SMALL: &str = "Array too small ({} < {})";
/// The array node's maximum element count.
pub const ARRAY_TOO_LARGE: &str = "Array too large ({} > {})";
/// The JSON reader failed on the file.
pub const PARSE_ERROR: &str = "parse error:\n{}";

/// A vanilla text with its arguments, as the code fields take it.
#[allow(
    clippy::unnecessary_wraps,
    reason = "every code field is an `Option<VanillaText>`"
)]
pub fn text(template: &'static str, args: &[&str]) -> Option<VanillaText> {
    Some(VanillaText::Formatted {
        template,
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
    })
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

/// Whether two nodes hold one JSON value, whatever the spacing and the order of their members.
fn same_value(a: Node<'_>, b: Node<'_>) -> bool {
    match (
        parse_at::<facet_value::Value>(a.slice),
        parse_at::<facet_value::Value>(b.slice),
    ) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

/// One JSON value of the file, as its slice of the text (trimmed to the value's own tokens).
#[derive(Clone, Copy, Debug)]
pub struct Node<'a> {
    slice: JsonSlice<'a>,
}

impl<'a> Node<'a> {
    /// The value of a whole text (comments and whitespace around it skipped); `None` when the
    /// text is not JSON.
    pub fn root(text: &'a str) -> Option<Self> {
        Self::of(JsonSlice::new(text))
    }

    /// The value of a slice.
    pub fn of(slice: JsonSlice<'a>) -> Option<Self> {
        let raw: RawJson<'a> = parse_at(slice).ok()?;
        slice.nested(&raw).map(|slice| Self { slice })
    }

    /// The JSON type of the node, from its first byte.
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
    pub fn text(&self) -> &'a str {
        self.slice.text
    }

    /// The node's place in the file, saturated to `u32`.
    pub fn span(&self) -> Span {
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

// ---- member-name patterns ---------------------------------------------------------------------

/// A member name or pattern of a schema object, matched in full and ignoring ASCII case.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Pattern {
    /// A literal member name.
    Name(&'static str),
    /// `<prefix>[a-zA-Z0-9_.-]+` with an escaped dotted prefix (`animation\.`).
    Prefixed(&'static str, &'static str),
    /// `[a-zA-Z0-9_.-]+`.
    Identifier,
    /// `[0-9]+(\.[0-9]+)?`.
    Time,
    /// `[xyz]`.
    Axis,
}

impl Pattern {
    /// Whether `key` is a name this pattern accepts (names and prefixes compare without regard to
    /// case).
    pub fn matches(self, key: &str) -> bool {
        match self {
            Self::Name(name) => key.eq_ignore_ascii_case(name),
            Self::Prefixed(prefix, _) => {
                key.get(..prefix.len())
                    .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
                    && is_identifier(&key[prefix.len()..])
            }
            Self::Identifier => is_identifier(key),
            Self::Time => is_time_key(key),
            Self::Axis => matches!(key, "x" | "y" | "z" | "X" | "Y" | "Z"),
        }
    }

    /// The pattern as vanilla prints it in "Required child %s not found".
    pub fn text(self) -> &'static str {
        match self {
            Self::Name(name) => name,
            Self::Prefixed(_, regex) => regex,
            Self::Identifier => "[a-zA-Z0-9_.-]+",
            Self::Time => r"[0-9]+(\.[0-9]+)?",
            Self::Axis => "[xyz]",
        }
    }
}

/// `[a-zA-Z0-9_.-]+`, in full.
pub(crate) fn is_identifier(key: &str) -> bool {
    !key.is_empty()
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
}

/// `[0-9]+(\.[0-9]+)?`, in full.
pub(crate) fn is_time_key(key: &str) -> bool {
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    match key.split_once('.') {
        Some((whole, fraction)) => digits(whole) && digits(fraction),
        None => digits(key),
    }
}

/// vanilla's `atof` of a key that matched `is_time_key` (or any key: `atof` reads the longest
/// numeric prefix and gives 0 for none).
pub fn atof(key: &str) -> f32 {
    let end = key
        .char_indices()
        .take_while(|(i, c)| c.is_ascii_digit() || *c == '.' || (*i == 0 && matches!(c, '-' | '+')))
        .last()
        .map_or(0, |(i, c)| i + c.len_utf8());
    let mut prefix = &key[..end];
    // `atof("1.")` is 1; Rust's parser wants digits after the point.
    while !prefix.is_empty() && prefix.parse::<f32>().is_err() {
        prefix = &prefix[..prefix.len() - 1];
    }
    prefix.parse().unwrap_or(0.0)
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
    pub const fn name(name: &'static str) -> Self {
        Self {
            pattern: Pattern::Name(name),
            required: false,
        }
    }

    /// A required group of one fixed name.
    pub const fn required(name: &'static str) -> Self {
        Self {
            pattern: Pattern::Name(name),
            required: true,
        }
    }

    /// A group of any pattern.
    pub const fn pattern(pattern: Pattern, required: bool) -> Self {
        Self { pattern, required }
    }
}

/// What reading one member gave.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    /// The member was read.
    Ok,
    /// The member failed (its finding is reported).
    Failed,
    /// The member's only option refused it by its predicate: vanilla's search moves on and
    /// ends with "child '%s' not valid here.".
    NoOption,
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

impl From<bool> for Hit {
    fn from(ok: bool) -> Self {
        if ok { Self::Ok } else { Self::Failed }
    }
}

// ---- the walk ---------------------------------------------------------------------------------

/// The walk's state: the findings so far and the breadcrumb of the value being read.
#[derive(Debug, Default)]
pub struct Walk {
    /// The findings of the walk so far.
    pub diagnostics: Diagnostics,
    path: JsonPath,
    /// Member names spelled in another case than declared (controllers report them).
    pub non_canonical: Vec<(JsonPath, Span)>,
}

impl Walk {
    /// An empty walk, at the root.
    pub fn new() -> Self {
        Self::default()
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
        let mut finding = Diagnostic::error(code)
            .severity(severity)
            .at(self.path.clone());
        finding.location.span = span;
        self.diagnostics.push(finding);
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
    fn unknown(&mut self, key: &str, node: Node<'_>) {
        self.enter(key);
        self.error(
            DiagnosticCode::UnrecognizedField {
                text: text(CHILD_NOT_VALID, &[key]),
            },
            node.span(),
        );
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

    /// `"Value '<value>' not allowed"` (a bool outside its set) at the current path.
    pub fn not_allowed(&mut self, value: &str, node: Node<'_>) {
        self.error(
            DiagnosticCode::InvalidValue {
                text: text(VALUE_NOT_ALLOWED, &[value]),
            },
            node.span(),
        );
    }

    /// The members of an object node, in file order (a repeated key keeps its first position
    /// and its last value, as jsoncpp keeps the last).
    #[allow(clippy::unused_self, reason = "read like the other node accessors")]
    pub fn members<'a>(&self, node: Node<'a>) -> Option<Vec<(String, Node<'a>)>> {
        let map: IndexMap<String, RawJson<'a>> = parse_at(node.slice).ok()?;
        map.iter()
            .map(|(key, raw)| Some((key.clone(), node.nested(raw)?)))
            .collect()
    }

    /// The elements of an array node.
    #[allow(clippy::unused_self, reason = "read like the other node accessors")]
    fn elements<'a>(&self, node: Node<'a>) -> Option<Vec<Node<'a>>> {
        let raw: Vec<RawJson<'a>> = parse_at(node.slice).ok()?;
        raw.iter().map(|raw| node.nested(raw)).collect()
    }

    /// Walks an object node against its name groups: each member is handed to `on` with the index
    /// of the first group it matches; an unmatched member is "child '%s' not valid here.". When
    /// every member was read, each required group without a member is "Required child %s not
    /// found". `false` when anything failed. A member of a literal name that a later member of
    /// the same name in another case replaces is told as dropped.
    pub fn object<'a>(
        &mut self,
        node: Node<'a>,
        groups: &[Group],
        mut on: impl FnMut(&mut Self, usize, &str, Node<'a>) -> Hit,
    ) -> bool {
        let Some(members) = self.members(node) else {
            self.bad_type("object", node);
            return false;
        };
        let mut ok = true;
        let mut found = vec![false; groups.len()];
        // The member last read for each group of one literal name.
        let mut named: Vec<Option<(String, Span, Node<'a>)>> = vec![None; groups.len()];
        for (key, value) in members {
            let Some(index) = groups.iter().position(|g| g.pattern.matches(&key)) else {
                self.unknown(&key, value);
                ok = false;
                continue;
            };
            if matches!(groups[index].pattern, Pattern::Name(_))
                && let Some((earlier, span, earlier_value)) =
                    named[index].replace((key.clone(), value.span(), value))
                && !same_value(earlier_value, value)
            {
                // Two members that differ in case alone are one member: this one's value
                // replaces the earlier's, which is told where the file has it unless the
                // value is the same (a `DroppedField` information of the crate's own).
                self.enter(&earlier);
                self.push(
                    DiagnosticCode::DroppedField { text: None },
                    Severity::Info,
                    Some(span),
                );
                self.leave();
            }
            let spelled_otherwise = match groups[index].pattern {
                Pattern::Name(name) => key != name,
                Pattern::Prefixed(prefix, _) => !key.starts_with(prefix),
                _ => false,
            };
            if spelled_otherwise {
                let at = self.path.child(&key);
                self.non_canonical.push((at, value.span()));
            }
            self.enter(&key);
            let hit = on(self, index, &key, value);
            self.leave();
            match hit {
                Hit::Ok => found[index] = true,
                Hit::Failed => ok = false,
                Hit::NoOption => {
                    self.unknown(&key, value);
                    ok = false;
                }
            }
        }
        if ok {
            for (group, found) in groups.iter().zip(found) {
                if group.required && !found {
                    self.missing(group.pattern, node);
                    ok = false;
                }
            }
        }
        ok
    }

    /// The member count of an object node, for the object nodes with a child-count bound
    /// (`transitions[]` elements, `v1_8` `animations[]` elements): exactly `count` members, else
    /// "Node has too few/many children".
    pub fn child_count(&mut self, node: Node<'_>, count: usize) -> bool {
        let Some(members) = self.members(node) else {
            return true;
        };
        let n = members.len();
        let code = match n.cmp(&count) {
            std::cmp::Ordering::Less => {
                bedrock_addon_core::diagnostic::cereal::too_few_children(n, count)
            }
            std::cmp::Ordering::Greater => {
                bedrock_addon_core::diagnostic::cereal::too_many_children(n, count)
            }
            std::cmp::Ordering::Equal => return true,
        };
        self.error(code, node.span());
        false
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

    /// The kinds of the elements of an array node (none when the node is no array), for predicates
    /// over the whole array.
    pub fn kinds(&mut self, node: Node<'_>) -> Vec<Kind> {
        self.elements(node)
            .map(|e| e.iter().map(Node::kind).collect())
            .unwrap_or_default()
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
    pub fn number(&mut self, node: Node<'_>) -> Option<f64> {
        if node.kind() != Kind::Number {
            self.bad_type("float", node);
            return None;
        }
        parse_at::<f64>(node.slice).ok()
    }

    /// A number node as `f32`: the nearest float, or the largest finite one of its sign for
    /// a number beyond a float (with an `InvalidValue` warning at the number).
    #[allow(
        clippy::cast_possible_truncation,
        reason = "vanilla reads these members as float"
    )]
    pub fn float(&mut self, node: Node<'_>) -> Option<f32> {
        let nearest = self.number(node)? as f32;
        if nearest.is_infinite() {
            // A number beyond a float is the largest finite float of its sign, with an
            // information on its token; the file loads, and the written text reads back to the
            // same value.
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

    /// A schema array of exactly three `ExpressionNode`s (`addChildArray(name, 3, …)`).
    pub fn molang3(&mut self, node: Node<'_>) -> Option<[Molang; 3]> {
        if node.kind() != Kind::Array {
            self.bad_type("array", node);
            return None;
        }
        // The array is read once, for its length and for its elements.
        let elements = self.elements(node).unwrap_or_default();
        let n = elements.len();
        if n != 3 {
            let (template, op) = if n < 3 {
                (ARRAY_TOO_SMALL, n.to_string())
            } else {
                (ARRAY_TOO_LARGE, n.to_string())
            };
            self.error(
                DiagnosticCode::InvalidValue {
                    text: text(template, &[&op, "3"]),
                },
                node.span(),
            );
            return None;
        }
        let mut values = Vec::with_capacity(3);
        for (index, element) in elements.into_iter().enumerate() {
            self.enter_index(index);
            values.extend(self.molang(element));
            self.leave();
        }
        values.try_into().ok()
    }

    /// An array of strings (`on_entry`, `parameters`, timeline arrays).
    pub fn strings(&mut self, node: Node<'_>) -> Option<Vec<String>> {
        let mut out = Vec::new();
        let ok = self.array(node, |walk, _, element| {
            walk.string(element).map(|s| out.push(s)).is_some()
        });
        ok.then_some(out)
    }

    /// An object of `"<key>": float` members under a required pattern group.
    pub fn float_map(
        &mut self,
        node: Node<'_>,
        pattern: Pattern,
        required: bool,
    ) -> Option<IndexMap<String, f32>> {
        let mut map = IndexMap::new();
        let ok = self.object(
            node,
            &[Group::pattern(pattern, required)],
            |walk, _, key, v| walk.float(v).map(|x| map.insert(key.to_owned(), x)).into(),
        );
        ok.then_some(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_keys_match_in_full() {
        for key in ["0", "0.5", "12.25"] {
            assert!(is_time_key(key), "{key}");
        }
        for key in [".5", "-1", "1e2", "0.5 ", "1.", ""] {
            assert!(!is_time_key(key), "{key}");
        }
    }

    #[test]
    fn atof_reads_the_numeric_prefix() {
        assert!((atof("0.25") - 0.25).abs() < f32::EPSILON);
        assert!(atof("x").abs() < f32::EPSILON);
        assert!((atof("1.") - 1.0).abs() < f32::EPSILON);
    }

    /// What `molang3` says of an array, by its length and its elements: the findings as
    /// `code at path`.
    fn three(text: &str) -> (Option<[Molang; 3]>, Vec<String>) {
        let mut walk = Walk::new();
        walk.enter("rotation");
        let read = walk.molang3(Node::root(text).unwrap());
        let findings = walk
            .diagnostics
            .iter()
            .map(|d| format!("{} at {}", d.code.name(), d.location.json_path))
            .collect();
        (read, findings)
    }

    #[test]
    fn an_array_of_three_expressions_is_told_of_by_its_length_then_by_its_elements() {
        let (read, findings) = three(r#"[1, "q.x", true]"#);
        assert!(read.is_some() && findings.is_empty(), "{findings:?}");
        // The length first: no element of an array of another length is read.
        for text in ["[]", "[1, {}]", "[1, {}, [], 4]", "7"] {
            let (read, findings) = three(text);
            let code = if text == "7" {
                "InvalidType"
            } else {
                "InvalidValue"
            };
            assert!(read.is_none(), "{text}");
            assert_eq!(findings, [format!("{code} at rotation")], "{text}");
        }
        // Then every element that is no expression, each at its index.
        let (read, findings) = three("[{}, 2, []]");
        assert!(read.is_none());
        assert_eq!(
            findings,
            ["InvalidType at rotation/0", "InvalidType at rotation/2"]
        );
    }

    /// An array of three expressions is read once, for its length and for its elements, not
    /// once for each. The test compares two times rather than bounding growth: `molang3` on an
    /// array of three large elements (objects, each refused without being read) against one read of
    /// the same array's elements (`kinds`), each the best of seven taken in turn, required to stay
    /// under one and a half times it (2.0 with the two reads, 1.0 with the one), and measured once
    /// more before a time over the bound is believed.
    #[test]
    fn an_array_of_three_expressions_is_read_once() {
        use bedrock_addon_core::test_cost::{alone_for, time_of};
        let element = format!(r#"{{"k": [{}0]}}"#, "1, ".repeat(40_000));
        let text = format!("[{element}, {element}, {element}]");
        let node = Node::root(&text).unwrap();
        let mut ratios = Vec::new();
        for _attempt in 0..2 {
            let (three, once) = alone_for(|| {
                let mut best = (std::time::Duration::MAX, std::time::Duration::MAX);
                for _ in 0..7 {
                    let mut walk = Walk::new();
                    best.0 = best
                        .0
                        .min(time_of(|| assert!(walk.molang3(node).is_none())));
                    best.1 = best
                        .1
                        .min(time_of(|| assert_eq!(walk.kinds(node).len(), 3)));
                }
                best
            });
            let ratio = three.as_secs_f64() / once.as_secs_f64().max(1e-9);
            eprintln!(
                "cost: three expressions {three:?}, one read of the array {once:?}, ratio {ratio:.2}"
            );
            if ratio < 1.5 {
                return;
            }
            ratios.push(ratio);
        }
        panic!("an array of three expressions took {ratios:?} times one read of it, twice");
    }

    #[test]
    fn prefixed_names_ignore_case_and_match_in_full() {
        let p = Pattern::Prefixed("animation.", r"animation\.[a-zA-Z0-9_.-]+");
        assert!(p.matches("Animation.Foo"));
        assert!(!p.matches("animation."));
        assert!(!p.matches("animation:foo"));
        assert!(!p.matches("animation.a b"));
    }

    /// The keys `object` hands its callback, in the order it hands them, and the findings.
    fn walked(text: &str, groups: &[Group], read: Hit) -> (bool, Vec<String>, Vec<String>) {
        let mut walk = Walk::new();
        let mut keys = Vec::new();
        let ok = walk.object(Node::root(text).unwrap(), groups, |_, _, key, _| {
            keys.push(key.to_owned());
            read
        });
        let findings = walk
            .diagnostics
            .iter()
            .map(|d| format!("{:?} {} at {}", d.severity, d.code, d.location.json_path))
            .collect();
        (ok, keys, findings)
    }

    /// W1: the members of an object are walked in the file's order.
    #[test]
    fn members_are_walked_in_document_order() {
        let groups = [Group::name("a"), Group::name("b"), Group::name("c")];
        let (ok, keys, _) = walked(r#"{"b": 1, "a": 2, "c": 3}"#, &groups, Hit::Ok);
        assert!(ok);
        assert_eq!(keys, ["b", "a", "c"]);
    }

    /// W2: a member whose read fails fails the object, though it raised no finding, and the
    /// required check is then not made.
    #[test]
    fn a_member_failed_without_a_finding_fails_the_object_and_skips_the_required_check() {
        let groups = [Group::name("a"), Group::required("b")];
        let (ok, keys, findings) = walked(r#"{"a": 1}"#, &groups, Hit::Failed);
        assert!(!ok);
        assert_eq!(keys, ["a"]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    /// W3: a number a float does not hold is read without a `LossyConversion` information.
    #[test]
    fn a_float_that_narrows_is_not_told_as_lossy() {
        let mut walk = Walk::new();
        walk.enter("x");
        assert_eq!(
            walk.float(Node::root("0.123456789012").unwrap()),
            Some(0.123_456_79)
        );
        assert!(walk.diagnostics.is_empty());
    }
}
