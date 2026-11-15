//! Components one at a time: each entry of a `components` object is parsed from its own slice,
//! so a failing one is dropped and the rest survive; and the generic per-component checks on
//! the newest struct (release gates, deprecation, area, superseded versions).

use facet::Facet;
use facet_json::RawJson;
use indexmap::IndexMap;

use super::payload::{failure_finding, join, span_of, tell_unknown};
use super::{
    ComponentTable, ContextArea, ConvertMode, ExperimentalField, Family, FieldGate, FieldMeta,
};
use crate::diagnostic::codes as core_codes;
use crate::diagnostic::{
    Class, Code, Diagnostic, Diagnostics, JsonPath, Location, Phase, Severity,
};
use crate::document::{ComponentMap, ComponentRef, ComponentSet, CustomComponent};
use crate::json::{JsonSlice, NullRule, ScalarRead, Value, parse_with_rules, reject_nulls};
use crate::load::{Experiment, LoadContext, LoadRequirement};
use crate::version::FormatVersion;

/// Parses one typed component from its slice: `T` from the text (its plain scalars read by the
/// slice's scalar rules, `LoadContext::scalar_rules`; strict wrappers decide what coerces, so
/// `mode` is the caller's to honour), its unknown members kept in `d`'s table (by paths
/// relative to the component, which the component-map parser places; which structs keep them
/// is `parse_payload`'s rule) and told as `parse_payload` tells them (an `UnrecognizedField`
/// graded by the slice's reading, or the word a struct states). A failed parse (a member of the wrong type, `0x8`, or a required member missing,
/// `0x80`) is one finding: in a cereal slice of a document-input family
/// ([`LoadContext::strict_components`]) an error, which rejects the whole document under both
/// masks; elsewhere (`JsonCpp` slices, hand-written legacy slices) an error too. A legacy
/// block whose component fails to parse logs "Failed to parse block '%s'" at level 3 and the
/// game refuses the whole block, and the `JsonCpp` readers fail a value of another type with
/// code `0x8`, an error under both masks. Findings are relative to the component; the
/// component-map parser places them. The usual `parse` of a `#[derive(Components)]` row:
/// `parse = versioning::parse_component::<Friction, ItemComponent>`.
///
/// A row with `shorthand = scalar` whose payload is a struct of one member
/// ([`ShorthandShape::OneField`](crate::schema::ShorthandShape)) also reads a payload that is
/// no object (`5` for `{"value": 5}`): the text is read as that member, by the slice's scalar
/// rules and its null rule (a bare `null` is the member's `null`: a type error under
/// `Rejected`, the member left out under `Absent`), its findings at the component. Under
/// [`ScalarRules::default_and_tell`](crate::json::ScalarRules) a bare scalar of the wrong
/// type is the member's: blanked and told, and the component kept with the member's default. A failed read spans the text as
/// written; a scalar no arm of an untagged member takes is told, as every such refusal at the
/// door ([`ScalarFailure::Refused`](crate::json::ScalarFailure)), without a span.
/// An untagged enum or a transparent newtype reads the scalar itself, so the row's shorthand
/// changes nothing there.
pub fn parse_component<T, C>(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<C>
where
    T: for<'a> Facet<'a> + Into<C>,
    C: crate::schema::ShorthandRows,
{
    let _ = mode;
    if let Some(key) = shorthand_member::<T, C>(slice) {
        return parse_shorthand::<T>(slice, key, cx, d).map(Into::into);
    }
    let rule = cx.null_rule;
    let ScalarRead {
        value,
        told,
        json,
        absent,
    } = match parse_with_rules::<T>(slice, rule, cx.scalar_rules) {
        Ok(read) => read,
        Err(failure) => {
            // Both readers fail the slice; `strict_components` only names the input.
            let _ = cx.strict_components;
            d.push(failure_finding(&failure, &JsonPath::new(), slice));
            return None;
        }
    };
    // Under the `Rejected` null rule a `null` where the component reads a value is a type
    // error, which rejects the document.
    if rule == Some(NullRule::Rejected) && reject_nulls(&value, slice, &JsonPath::new(), d) {
        return None;
    }
    if let Some(json) = json {
        // Each member read as absent (rule N1; the pre-pass left it out of `json`) is marked:
        // the write-back writes its default.
        crate::json::tell_null_members(&absent, &JsonPath::new(), d);
        let mut kept = crate::document::Unknown::new();
        let unknown = crate::json::door_unknown_members(&value, &json, cx.keeps_unknown, &mut kept);
        d.unknown_mut().absorb(&JsonPath::new(), kept);
        tell_unknown(unknown, &JsonPath::new(), cx, d);
        // A scalar its member read as another value is told where it is.
        for told in told {
            d.push(told.finding(&JsonPath::new()));
        }
    }
    Some(value.into())
}

/// Reads one typed component from its slice on the new reader ([`FromJson`](crate::schema::FromJson),
/// member by member): what [`parse_component`] does, the same signature, so a row's `parse`
/// takes `read_component::<T, C>` where it took `parse_component::<T, C>`. `T` read under the
/// slice's reading by the typed door's profile
/// ([`Profile::FIRST_FAILURE`](crate::json::kit::Profile::FIRST_FAILURE)), on a stack its depth
/// can take, every finding relative to the component (the component-map parser places them)
/// and spanning its value; the unknown members it keeps in `d`'s table, relative to it.
///
/// A row with `shorthand = scalar` whose payload is a struct of one member reads a payload
/// that is no object as that member, as `parse_component` does: by the slice's scalar and null
/// rules, its findings at the component, spanning the text as written.
///
/// A component whose JSON is of the wrong kind for its type's root (an array or a scalar for a
/// struct, an array for a scalar or a one-member shorthand) is told as the typed door tells it,
/// `core/syntax` ("unexpected token: got X, expected Y") at the component: the oracle
/// `a_component_of_the_wrong_json_kind_is_the_door_s_syntax_error` pins the words.
pub fn read_component<T, C>(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<C>
where
    T: crate::schema::FromJson + for<'a> Facet<'a> + Into<C>,
    C: crate::schema::ShorthandRows,
{
    use crate::json::kit::Node;
    let _ = mode;
    let parsed = parsed_or_told(slice, d)?;
    let shorthand = shorthand_member::<T, C>(slice);
    let member = shorthand.map(|key| member_kind(<T as Facet<'static>>::SHAPE, key));
    if let Some(finding) = door_syntax::<T>(&parsed.value, member, cx, span_of(slice)) {
        d.push(finding);
        return None;
    }
    if let Some(key) = shorthand {
        // The bare scalar is the member `key` of the object form; what is told of it is told
        // at the component, spanning the text.
        let mut object = facet_value::VObject::new();
        object.insert(key, parsed.value.clone());
        let wrapped = Value::from(object);
        let mut found = Diagnostics::default();
        let value = {
            let mut read = door_read(cx, &mut found);
            crate::schema::read_document::<T>(&Node::new(&wrapped, JsonPath::new()), &mut read)
        };
        let kept = found.take_unknown();
        for mut finding in found {
            finding.location.json_path = below_member(&finding.location.json_path);
            finding.location.span = finding.location.span.or(Some(span_of(slice)));
            d.push(finding);
        }
        d.unknown_mut().absorb(&JsonPath::new(), kept);
        return value.map(Into::into);
    }
    let node = Node::parsed(&parsed, JsonPath::new());
    let mut read = door_read(cx, d);
    crate::schema::read_document::<T>(&node, &mut read).map(Into::into)
}

/// Reads one value `T` from `slice` on the new reader, as [`read_component`] reads a component
/// that has no shorthand (a value of the wrong JSON kind for `T`'s root is the door's
/// `core/syntax` there as well): under the context's reading by the typed door's profile
/// ([`Profile::FIRST_FAILURE`](crate::json::kit::Profile::FIRST_FAILURE)), every finding
/// relative to the value and spanning it, the unknown members it keeps in `d`'s table, relative
/// to it. For a family's row that reads a form of its own beside the typed one (a bare key
/// where the object form is typed): the bare scalar is a `T` read here.
pub fn read_slice<T: crate::schema::FromJson>(
    slice: JsonSlice<'_>,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<T> {
    use crate::json::kit::Node;
    let parsed = parsed_or_told(slice, d)?;
    if let Some(finding) = door_syntax::<T>(&parsed.value, None, cx, span_of(slice)) {
        d.push(finding);
        return None;
    }
    let node = Node::parsed(&parsed, JsonPath::new());
    let mut read = door_read(cx, d);
    crate::schema::read_document::<T>(&node, &mut read)
}

/// The text of `slice` parsed with its spans; one that does not parse is told as the door tells
/// it (not reached for a component the payload's parse has read).
fn parsed_or_told<'t>(
    slice: JsonSlice<'t>,
    d: &mut Diagnostics,
) -> Option<crate::json::parse::Parsed<'t>> {
    let parsed = crate::json::parse::parse_slice(slice, crate::json::parse::Options::default());
    if parsed.is_err()
        && let Err(error) = crate::json::parse_at::<Value>(slice)
    {
        d.push(super::payload::slice_error(
            &error,
            &JsonPath::new(),
            slice,
            Severity::Error,
        ));
    }
    parsed.ok()
}

/// The read of a component or a bare value of the context: its reading, the typed door's
/// profile, the unknown severity of the slice and the document's format.
fn door_read<'d>(cx: &'d LoadContext, d: &'d mut Diagnostics) -> crate::json::kit::Read<'d> {
    use crate::json::kit::{Profile, Read};
    Read::new(cx.reading, d)
        .unknown_severity(cx.grade(Class::Unknown))
        .profile(Profile::FIRST_FAILURE)
        .load(cx)
        .format(cx.document_format)
}

/// The finding of a component whose JSON is of another kind than its type's root, told as the
/// typed door told it: `core/syntax`, "unexpected token: got X, expected Y", the malformed input
/// of facet's own read, where the reader's wrong type names no member. It is told when:
///
/// - `T` is a struct (`T::EXPECTED` an object) and the value is no object, `null` included;
/// - `T` reads a list (`T::EXPECTED` an array) and the value is no array;
/// - `T` reads a scalar (a transparent newtype) and the value is an array, or an empty object
///   (an object with a member is told at the member: a wrong type);
/// - the row is a one-member shorthand (`member` is the member's kind) of a scalar member and
///   the value is an array; of a unit enum, whatever the scalar rule says.
///
/// An array given to a scalar is a refused scalar under `DefaultAndTell` (the component is kept
/// with the member's default and the refusal told), and a wrong type at its first element under
/// strict scalars, not this.
fn door_syntax<T: crate::schema::FromJson>(
    value: &Value,
    member: Option<MemberKind>,
    cx: &LoadContext,
    span: crate::diagnostic::Span,
) -> Option<Diagnostic> {
    use crate::json::kit::Expected;
    // A scalar judged by the slice's rule (any but the plain typed reader's) reads an array
    // through its elements (a wrong type at the first), and a refused scalar under
    // `DefaultAndTell` is the scalar rule's.
    let refused = cx.reading.on_member_failure == super::MemberFailure::DefaultAndTell
        || cx.reading.scalars != super::Scalars::ConvertAndTell;
    let scalar = match member {
        Some(kind) => kind == MemberKind::Scalar,
        None => matches!(
            T::EXPECTED,
            Some(Expected::Number | Expected::Bool | Expected::String)
        ),
    };
    let (got, expected) = if member.is_none() && T::EXPECTED == Some(Expected::Object) {
        if value.is_object() {
            return None;
        }
        let got = if value.is_array() {
            "sequence start"
        } else {
            "scalar"
        };
        (got, "struct start")
    } else if member.is_none() && T::EXPECTED == Some(Expected::Array) {
        if value.is_array() {
            return None;
        }
        // Facet's words: a scalar is told by the parser, an object by the list's own reader.
        if value.is_object() {
            ("object", "array")
        } else {
            ("scalar", "sequence start")
        }
    } else if scalar && value.is_array() && !refused {
        ("sequence start", "scalar value")
    } else if member == Some(MemberKind::UnitEnum) && value.is_array() {
        // Facet's enum read: a name or an object, whatever the slice's scalar rule says.
        ("sequence start", "string or struct for enum")
    } else if scalar
        && member.is_none()
        && value
            .as_object()
            .is_some_and(facet_value::VObject::is_empty)
    {
        (
            "node without argument",
            "scalar value or node with argument",
        )
    } else {
        return None;
    };
    let mut finding = Diagnostic::of(crate::diagnostic::builders::syntax(format!(
        "unexpected token: got {got}, expected {expected}"
    )))
    .at(JsonPath::new());
    finding.location.span = Some(span);
    Some(finding)
}

/// What facet reads a one-member shorthand's member as ([`member_kind`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MemberKind {
    /// One token: a scalar, an `Option` or a transparent wrapper of one.
    Scalar,
    /// An enum of names only (every variant a unit): a name, or an object of one member.
    UnitEnum,
    /// Anything else.
    Other,
}

/// What the member `key` of the struct `shape` is read as ([`MemberKind`]).
fn member_kind(shape: &'static facet::Shape, key: &str) -> MemberKind {
    let facet::Type::User(facet::UserType::Struct(st)) = shape.ty else {
        return MemberKind::Other;
    };
    let Some(field) = st.fields.iter().find(|f| f.effective_name() == key) else {
        return MemberKind::Other;
    };
    let mut shape = field.shape();
    loop {
        shape = match shape.def {
            facet::Def::Option(option) => option.t(),
            _ => match shape.inner {
                Some(inner) if shape.is_transparent() => inner,
                _ => break,
            },
        };
    }
    if shape.effective_proxy(None).is_some() || matches!(shape.def, facet::Def::Scalar) {
        MemberKind::Scalar
    } else if let facet::Type::User(facet::UserType::Enum(kind)) = shape.ty
        && !shape.is_untagged()
        && kind.variants.iter().all(|v| v.data.fields.is_empty())
    {
        MemberKind::UnitEnum
    } else {
        MemberKind::Other
    }
}

/// The member a bare scalar of `slice` stands for: `T` is the payload of a `shorthand =
/// scalar` row of `C`, a struct of one member, and the text is no object.
fn shorthand_member<T, C>(slice: JsonSlice<'_>) -> Option<&'static str>
where
    T: for<'a> Facet<'a>,
    C: crate::schema::ShorthandRows,
{
    let shape = <T as Facet<'static>>::SHAPE;
    C::shorthand_key(shape)?;
    let crate::schema::ShorthandShape::OneField { key } = crate::schema::shorthand_shape(shape)?
    else {
        return None;
    };
    (!slice.opens_an_object()).then_some(key)
}

/// Reads the bare scalar of `slice` as the member `key` of `T`: the text is read inside
/// `{"key": …}`, and what is told of the member is told at the component, spanning the text
/// as the file has it.
fn parse_shorthand<T>(
    slice: JsonSlice<'_>,
    key: &str,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<T>
where
    T: for<'a> Facet<'a>,
{
    use crate::json::ScalarFailure;
    let mut prefix = String::from("{");
    prefix.push_str(&crate::json::to_json(&key).ok()?);
    prefix.push(':');
    // A line comment at the end of the text ends before the closing brace.
    let wrapped = format!("{prefix}{}\n}}", slice.text);
    let inner = JsonSlice {
        text: &wrapped,
        offset: slice.offset,
    };
    // The member's own place is the text's: a span inside it moves back by the prefix, any
    // other (the object around it) is the text's whole.
    let start = slice.offset + prefix.len();
    let end = start + slice.text.len();
    let unwrap = |span: crate::diagnostic::Span| {
        let (offset, len) = (span.offset as usize, span.len as usize);
        if offset >= start && offset + len <= end {
            crate::diagnostic::Span {
                offset: span.offset - u32::try_from(prefix.len()).unwrap_or(u32::MAX),
                len: span.len,
            }
        } else {
            super::payload::span_of(slice)
        }
    };
    // A bare `null` is the member's `null`: the slice's null rule reads it as the object
    // form's.
    let rule = cx.null_rule;
    match parse_with_rules::<T>(inner, rule, cx.scalar_rules) {
        Ok(read) => {
            let mut found = Diagnostics::default();
            if rule == Some(NullRule::Rejected)
                && reject_nulls(&read.value, inner, &JsonPath::new(), &mut found)
            {
                for mut finding in found {
                    finding.location.json_path = below_member(&finding.location.json_path);
                    finding.location.span = finding.location.span.map(unwrap);
                    d.push(finding);
                }
                return None;
            }
            let absent: Vec<JsonPath> = read.absent.iter().map(below_member).collect();
            crate::json::tell_null_members(&absent, &JsonPath::new(), d);
            for mut told in read.told {
                told.path = below_member(&told.path);
                d.push(told.finding(&JsonPath::new()));
            }
            Some(read.value)
        }
        Err(failure) => {
            let failure = match failure {
                ScalarFailure::Read(mut error) => {
                    error.path = below_member(&error.path);
                    error.span = unwrap(error.span);
                    ScalarFailure::Read(error)
                }
                ScalarFailure::Refused(mut refused) => {
                    refused.misread.path = below_member(&refused.misread.path);
                    // Kept right for `spanned_finding`; the door's finding has no span.
                    refused.span = refused.span.map(unwrap);
                    ScalarFailure::Refused(refused)
                }
            };
            d.push(failure_finding(&failure, &JsonPath::new(), slice));
            None
        }
    }
}

/// `path` without its first step, the member a bare scalar was read as.
fn below_member(path: &JsonPath) -> JsonPath {
    let mut out = JsonPath::new();
    for segment in path.segments().skip(1) {
        match segment {
            crate::diagnostic::JsonPathSegment::Key(key) => out.push(key),
            crate::diagnostic::JsonPathSegment::Index(index) => out.push_index(index),
        }
    }
    out
}

/// Parses a `components` object held as raw slices (`entries`, borrowed from `within`, the
/// payload's text) into family `F`'s component map, in file order.
///
/// A key of `F`'s table is parsed by its row's `parse`; the findings it reports are placed below
/// `at/<key>` (`at` is the object's path in the file). A row whose value has no typed key (an
/// entity's dynamic keys) keeps the JSON as a dynamic entry. A key `is_custom` accepts is a
/// custom component, kept as written. Any other key is kept for write-back and reported as an
/// `UnrecognizedComponent`, graded by the slice's reading (`LoadContext::grade`).
pub fn parse_component_map<'a, F: Family>(
    entries: &IndexMap<String, RawJson<'a>>,
    within: JsonSlice<'a>,
    at: &JsonPath,
    mode: ConvertMode,
    is_custom: fn(&str) -> bool,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> ComponentMap<F::Component> {
    let table = table::<F>();
    let mut map = ComponentMap::new();
    for (key, raw) in entries {
        let slice = within.nested(raw).unwrap_or(JsonSlice {
            text: raw.as_str(),
            offset: within.offset,
        });
        let json = || crate::json::from_str_jsonc::<Value>(slice.text).unwrap_or(Value::NULL);
        let entry = Entry {
            key,
            slice,
            at,
            mode,
            is_custom,
        };
        entry.place(table, json, cx, &mut map, d);
    }
    map
}

/// What a family says of a key of its `components` object before the rows read it
/// ([`read_component_map_by`]).
#[derive(Debug)]
pub enum KeyGate {
    /// The row reads it (or it is a custom or an unknown component).
    Read,
    /// The slice refuses the key: told by the finding at the key, the component left out.
    Refuse(Diagnostic),
    /// The slice has no row for the key: told by the finding at the key, after the rows' own,
    /// and kept as written, an unknown component.
    Unread(Diagnostic),
    /// The slice refuses the key: told by the finding at the key where the file has it, among
    /// the rows' own, the component left out.
    Drop(Diagnostic),
}

/// Reads a `components` object at `node` into family `F`'s component map, in file order, on
/// the new reader: what [`parse_component_map`] does with the raw slices of the typed door,
/// from the node's members (each read by its row's `parse`, which is
/// [`read_component`] for a converted component, from the member's text), its findings told
/// to `read` below the node's path, the unknown members the rows keep in its table. A
/// component that fails is dropped alone; a key `is_custom` takes is a custom component; any
/// other is an unrecognized component, kept for write-back. The rows read in the read's load
/// ([`Read::load`](crate::json::kit::Read::load)), or in a detached one under the read's
/// reading where it has none. `None` (a wrong type told) when the node is no object.
///
/// The family judges each key first with `gate` ([`KeyGate`]): the refused ones are told before
/// any row reads, the ones with no row in the slice after the last row, the dropped ones where
/// the file has them; `|_| KeyGate::Read` reads every key by its row. A family whose document
/// holds its components as a map reads that field with a function of its own that names its
/// family: `schema::read_with = components`, where `components` is
/// `read_component_map_by::<EntityFamily>(node, read, ConvertMode::Strict, is_custom, gate)`.
pub fn read_component_map_by<F: Family>(
    node: &crate::json::kit::Node<'_>,
    read: &mut crate::json::kit::Read<'_>,
    mode: ConvertMode,
    is_custom: fn(&str) -> bool,
    gate: impl Fn(&str) -> KeyGate,
) -> Option<ComponentMap<F::Component>> {
    if read.stopped() {
        return None;
    }
    let members = read.object(node)?;
    let detached;
    let cx = if let Some(cx) = read.load_context() {
        cx
    } else {
        let mut cx = LoadContext::detached();
        cx.reading = read.reading();
        detached = cx;
        &detached
    };
    // A member `schema::since_format` a later version than the document's is no member of its
    // component: the rows read knowing the document's format.
    let format = read.known_format();
    let versioned;
    let cx = if format.is_some() && cx.document_format != format {
        let mut cx = cx.clone();
        cx.document_format = format;
        versioned = cx;
        &versioned
    } else {
        cx
    };
    let table = table::<F>();
    let mut map = ComponentMap::new();
    let mut found = Diagnostics::default();
    let (mut rows, mut unread) = (Vec::new(), Vec::new());
    for (key, member) in members.iter() {
        let (path, span) = (node.path().child(key), member.span());
        let at = |mut finding: Diagnostic| {
            finding.location.json_path = path;
            finding.location.span = span;
            finding
        };
        match gate(key) {
            KeyGate::Read => rows.push((key, member, None)),
            KeyGate::Refuse(finding) => read.push(at(finding)),
            KeyGate::Unread(finding) => unread.push((key, member, at(finding))),
            KeyGate::Drop(finding) => rows.push((key, member, Some(at(finding)))),
        }
    }
    for (key, member, dropped) in rows {
        if let Some(finding) = dropped {
            found.push(finding);
            continue;
        }
        // A member without a text (a node made without one) is read from its value written out.
        let written;
        let slice = if let Some(slice) = member.text() {
            slice
        } else {
            written = crate::json::to_json(member.value()).unwrap_or_default();
            JsonSlice::new(&written)
        };
        let key = key.to_owned();
        let entry = Entry {
            key: &key,
            slice,
            at: node.path(),
            mode,
            is_custom,
        };
        entry.place(table, || member.value().clone(), cx, &mut map, &mut found);
    }
    let kept = found.take_unknown();
    for finding in found {
        read.defer(crate::json::kit::Pass::Misread, finding);
    }
    for (key, member, finding) in unread {
        read.push(finding);
        map.insert_unknown(key, member.value().clone());
    }
    read.findings_mut()
        .unknown_mut()
        .absorb(&JsonPath::new(), kept);
    Some(map)
}

/// One member of a `components` object, for the read of a component map.
struct Entry<'a> {
    key: &'a String,
    slice: JsonSlice<'a>,
    /// The object's path in the file.
    at: &'a JsonPath,
    mode: ConvertMode,
    is_custom: fn(&str) -> bool,
}

impl Entry<'_> {
    /// Reads the member into `map`: a key of `table` by its row's `parse`, the findings it
    /// tells placed below `at/<key>`; a row whose value has no typed key (an entity's dynamic
    /// keys) keeps the JSON (`json`) as a dynamic entry; a key `is_custom` accepts is a custom
    /// component, kept as written; any other key is kept for write-back and reported as an
    /// `UnrecognizedComponent`, graded by the slice's reading (`LoadContext::grade`).
    fn place<F: Family>(
        &self,
        table: &ComponentTable<F>,
        json: impl FnOnce() -> Value,
        cx: &LoadContext,
        map: &mut ComponentMap<F::Component>,
        d: &mut Diagnostics,
    ) {
        let (key, slice) = (self.key, self.slice);
        let mut path = self.at.clone();
        path.push(key);
        if let Some(meta) = table.get(key) {
            let mut found = Diagnostics::default();
            let value = (meta.parse)(slice, self.mode, cx, &mut found);
            let kept = found.take_unknown();
            // A component that goes for what is told of a member BELOW it is told at the
            // component too: the member's finding does not say that its siblings go with
            // it. An error refuses the file, and a finding at the component speaks for it.
            let told_at_it = found.has_errors()
                || found
                    .iter()
                    .any(|finding| finding.location.json_path.is_empty());
            for mut diagnostic in found {
                diagnostic.location.json_path = join(&path, &diagnostic.location.json_path);
                d.push(diagnostic);
            }
            match value {
                Some(value) if value.name().is_some() => {
                    // The component's unknown members, below its key; a component kept as
                    // JSON (or not at all) has none apart from what it holds.
                    d.unknown_mut().absorb(&path, kept);
                    map.insert_typed(value, false);
                }
                Some(_) => map.insert_dynamic(key, json()),
                None if told_at_it => {}
                // The game says nothing here, so the crate reports a read that failed without a
                // finding of its own as an information.
                None => d.push(Diagnostic::new(
                    Code::from(&core_codes::DROPPED_COMPONENT),
                    Severity::Info,
                    Phase::Parse,
                    Location::new(None, None, path, Some(span_of(slice))),
                )),
            }
        } else if (self.is_custom)(key) {
            map.insert_custom(CustomComponent {
                name: key.clone(),
                parameters: json(),
                from_v1_list: false,
            });
        } else {
            map.insert_unknown(key, json());
            d.push(Diagnostic::new(
                // Cereal's own text for a component the schema does not list: level 2, an
                // error under a strict slice's mask.
                crate::diagnostic::cereal::component_not_in_schema(),
                cx.grade(Class::Unknown),
                Phase::Parse,
                Location::new(None, None, path, Some(span_of(slice))),
            ));
        }
    }
}

/// One component map of the newest struct, for the generic per-component checks.
pub struct ComponentSection<'a, C: ComponentSet> {
    /// Its path below the payload (`components`, `permutations/2/components`).
    pub at: JsonPath,
    /// A failed requirement skips the section (a permutation) instead of rejecting the
    /// document (the base components).
    pub optional: bool,
    /// The map.
    pub map: &'a mut ComponentMap<C>,
}

impl<'a, C: ComponentSet> ComponentSection<'a, C> {
    /// The section of `map` at `at` (below the payload); `optional` for a permutation.
    pub fn new(at: JsonPath, optional: bool, map: &'a mut ComponentMap<C>) -> Self {
        Self { at, optional, map }
    }
}

/// The generic per-component checks on one map of the newest struct (the game's release-gate
/// check, compared against `original_version`), at `at` in the file. `Custom` entries and
/// `synthesized` ones bypass the table.
///
/// - Gates: allowed when `released` is `Some(r)` and `original_version >= r`; otherwise every
///   entry of `requires` must be satisfied, and an empty `requires` is no gate at all. A
///   failed requirement is `MissingToggles` with the text of its kind, an error (the document
///   is not loaded) for a base section, a warning for an optional one; the function then
///   returns `false`.
/// - Deprecation: at or above `deprecated_since`, `DeprecatedMember` (the row's message, else
///   "%s has been deprecated and is not available in json format %s") and the component is
///   dropped.
/// - Area: a `Vanilla` or `Internal` component outside a base-game pack is an
///   `UnrecognizedComponent` warning and is dropped.
/// - `supersedes`: a map holding both versions of a component is a
///   `DuplicateComponentVersion` warning and the older one is dropped.
pub fn validate_components<F: Family>(
    section: ComponentSection<'_, F::Component>,
    original_version: FormatVersion,
    cx: &LoadContext,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> bool {
    let what = F::COMPONENT_NOUN;
    let ComponentSection {
        at: rel,
        optional,
        map,
    } = section;
    let at = join(at, &rel);
    let table = table::<F>();
    // An optional section's gate refuses the document too: a block permutation's toggle
    // lines are logged at level 3 and a failing permutation fails the component load, so the
    // whole block is refused.
    let _ = optional;
    let gate_severity = Severity::Error;
    // A deprecated or out-of-area component is reported like a component missing from the
    // schema: level 2 under the unstrict mask, an error under the strict one.
    let unknown_severity = unknown_severity::<F>(original_version, cx.source);
    let mut passed = true;
    let mut dropped: Vec<String> = Vec::new();
    for entry in map.iter() {
        let name = match entry {
            ComponentRef::Typed {
                synthesized: true, ..
            }
            | ComponentRef::Custom(_)
            | ComponentRef::Unknown { .. } => continue,
            ComponentRef::Typed { name, .. } | ComponentRef::Dynamic { name, .. } => name,
        };
        let Some(meta) = table.get(name) else {
            continue;
        };
        let mut path = at.clone();
        path.push(name);
        let released = meta.released.is_some_and(|r| original_version >= r);
        if !released && let Some(failed) = meta.requires.iter().find(|r| !r.satisfied(cx)) {
            passed = false;
            d.push(finding(
                requirement_code(*failed, what, name, meta.released),
                gate_severity,
                path.clone(),
            ));
        }
        if let Some(since) = meta.deprecated_since
            && original_version >= since
        {
            let code = meta.deprecated_message.map_or_else(
                || {
                    core_codes::DEPRECATED_IN_FORMAT
                        .with([name.to_owned(), original_version.to_string()])
                },
                |message| core_codes::DEPRECATED_MEMBER_WORDED.with([message]),
            );
            // `minecraft:foot_size` at 1.16.100: the game logs the not-in-schema line at
            // level 2 and keeps the actor.
            d.push(finding(code, unknown_severity, path.clone()));
            dropped.push(name.to_owned());
        }
        if matches!(meta.area, ContextArea::Vanilla | ContextArea::Internal)
            && !cx.is_base_game_pack
        {
            d.push(finding(
                Code::from(&core_codes::UNRECOGNIZED_COMPONENT),
                unknown_severity,
                path.clone(),
            ));
            dropped.push(name.to_owned());
        }
        if let Some(old) = meta.supersedes
            && map.iter().any(|e| entry_name(&e) == old)
        {
            // The game's "Two versions of the same component" line belongs to the block data
            // fixer, and no block row has `supersedes`; an entity's rename logs nothing. The
            // crate reports the dropped older version as an information.
            d.push(finding(
                core_codes::DUPLICATE_COMPONENT_VERSION.with([old, name, name, old]),
                Severity::Info,
                path,
            ));
            dropped.push(old.to_owned());
        }
    }
    for name in &dropped {
        map.remove_named(name);
    }
    passed
}

/// The format-version field gate: every member of `fields` that `present` finds and
/// whose `released` is above `original_version` is an `UnsupportedVersion` error with "To use
/// field '%s.%s', use json format version %s or higher" (owner, then member) at the member below
/// `at` (a dotted name continues the path). `false` when one was found: the document is not
/// loaded, as vanilla's strict loaders refuse a member the version's struct does not bind.
pub fn check_fields_released(
    fields: &[FieldGate],
    present: impl Fn(&str) -> bool,
    owner: &str,
    original_version: FormatVersion,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> bool {
    let mut passed = true;
    for field in fields {
        if original_version >= field.released || !present(field.name) {
            continue;
        }
        passed = false;
        let mut path = at.clone();
        for part in field.name.split('.').filter(|p| !p.is_empty()) {
            path.push(part);
        }
        d.push(finding(
            use_format_version("field", &format!("{owner}.{}", field.name), field.released),
            Severity::Error,
            path,
        ));
    }
    passed
}

/// The experiment-gated fields of a beta struct (the game refuses such a member with result
/// code `0x40` while its experiment is off): every member of `fields`
/// that `present` finds while its experiment is off is a `MissingToggles` error naming the
/// experiment (`To use field '<owner>.<member>', turn on the '<experiment>' toggle in
/// settings`) at the member below `at` (a dotted name continues the path). `false` when one
/// was found: the document is not loaded.
pub fn check_experimental_fields(
    fields: &[ExperimentalField],
    present: impl Fn(&str) -> bool,
    owner: &str,
    at: &JsonPath,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> bool {
    let mut passed = true;
    for field in fields {
        if LoadRequirement::Experiment(field.experiment).satisfied(cx) || !present(field.name) {
            continue;
        }
        passed = false;
        let mut path = at.clone();
        for part in field.name.split('.').filter(|p| !p.is_empty()) {
            path.push(part);
        }
        d.push(Diagnostic::new(
            core_codes::TURN_ON_TOGGLE.with([
                "field".to_owned(),
                format!("{owner}.{}", field.name),
                field.experiment.save_name().unwrap_or_default().to_owned(),
            ]),
            Severity::Error,
            Phase::Parse,
            Location::new(None, None, path, None),
        ));
    }
    passed
}

/// "To use %s '%s', use json format version %s or higher".
fn use_format_version(what: &'static str, name: &str, version: FormatVersion) -> Code {
    core_codes::USE_FORMAT_VERSION.with([what.to_owned(), name.to_owned(), version.to_string()])
}

/// `FieldSinceEngine` warnings for every field of `fields` that `present` finds in a
/// component and that arrived after the pack's `min_engine_version` (an absent one counts as
/// older than every field). `at` is the component's path; a field's dotted name continues it.
pub fn check_fields_since_engine(
    fields: &[FieldMeta],
    present: impl Fn(&str) -> bool,
    at: &JsonPath,
    cx: &LoadContext,
    d: &mut Diagnostics,
) {
    let pack = cx
        .min_engine_version
        .as_ref()
        .map(|m| m.version.format_version());
    for field in fields {
        if pack.is_some_and(|p| p >= field.since_engine) || !present(field.name) {
            continue;
        }
        let mut path = at.clone();
        for part in field.name.split('.').filter(|p| !p.is_empty()) {
            path.push(part);
        }
        // The game has no line for a field newer than the pack, so the crate reports it as an
        // information.
        d.push(finding(
            core_codes::FIELD_SINCE_ENGINE.with([field.since_engine.to_string()]),
            Severity::Info,
            path,
        ));
    }
}

/// The code of a failed requirement: the legacy creator features name the version
/// that needs none; an experiment names its toggle, and the version from which none is
/// needed when the component has one. A requirement with no line of the game's (the base
/// game pack, a creator feature never released) is the plain [`core_codes::MISSING_TOGGLES`].
fn requirement_code(
    requirement: LoadRequirement,
    what: &'static str,
    name: &str,
    released: Option<FormatVersion>,
) -> Code {
    let toggle = |experiment: Experiment| experiment.save_name().unwrap_or_default().to_owned();
    let experiment = match requirement {
        LoadRequirement::LegacyCreatorFeatures => {
            return released.map_or_else(
                || Code::from(&core_codes::MISSING_TOGGLES),
                |version| {
                    core_codes::TOGGLE_USE_FORMAT_VERSION.with([
                        what.to_owned(),
                        name.to_owned(),
                        version.to_string(),
                    ])
                },
            );
        }
        LoadRequirement::BaseGamePack => return Code::from(&core_codes::MISSING_TOGGLES),
        LoadRequirement::BetaFeatures => Experiment::BetaApis,
        LoadRequirement::Experiment(experiment) => experiment,
    };
    match released {
        Some(version) => core_codes::TURN_ON_TOGGLE_BEFORE.with([
            what.to_owned(),
            name.to_owned(),
            version.to_string(),
            toggle(experiment),
        ]),
        None => {
            core_codes::TURN_ON_TOGGLE.with([what.to_owned(), name.to_owned(), toggle(experiment)])
        }
    }
}

/// The key of a map entry, for the `supersedes` lookup.
fn entry_name<'a, C>(entry: &ComponentRef<'a, C>) -> &'a str {
    match *entry {
        ComponentRef::Typed { name, .. }
        | ComponentRef::Dynamic { name, .. }
        | ComponentRef::Unknown { name, .. } => name,
        ComponentRef::Custom(custom) => &custom.name,
    }
}

/// A finding of the component checks at `path`.
/// `F`'s component table, where the pipeline reads components. A family whose component set
/// is inhabited (not [`NoComponents`](super::NoComponents), which has size 0) and that kept
/// the trait's empty default forgot to override [`Family::components`]: every component of
/// its documents would read as unknown. A debug assertion, since the table is static.
fn table<F: Family>() -> &'static ComponentTable<F> {
    let table = F::components();
    debug_assert!(
        size_of::<F::Component>() == 0 || !table.0.is_empty(),
        "family `{}` has components but an empty table: override `Family::components`",
        F::NAME
    );
    table
}

fn finding(code: Code, severity: Severity, path: JsonPath) -> Diagnostic {
    Diagnostic::new(
        code,
        severity,
        Phase::Validate,
        Location::new(None, None, path, None),
    )
}

/// The unknown severity of the struct that reads a document written at `version` from
/// `source`: what the unknown-component findings of the validate phase follow.
fn unknown_severity<F: Family>(
    version: FormatVersion,
    source: crate::load::DocumentSource,
) -> Severity {
    let input = crate::version::FormatVersionInput {
        version: Some(version),
        beta_literal: false,
        prerelease: false,
    };
    match super::range::select::<F>(&input) {
        super::range::Selection::Struct(i) => {
            super::reading::slice_reading::<F>(super::StructIndex::Stable(i), source).unknown
        }
        _ => Severity::Warning,
    }
}
