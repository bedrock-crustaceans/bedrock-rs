//! Typed parsing of a slice of a file: `JsonSlice` and `parse_at`.
//!
//! A version struct is not deserialised in one call: entries that must fail one by one
//! (`components`, the envelope payload) are held as `facet_json::RawJson` slices of the file text
//! and parsed on their own. `parse_at` reports a failure at
//! file offsets and with JSON member names.

use facet::{Def, Facet, Shape, Type, UserType};
use facet_json::{DeserializeError, RawJson};
use facet_path::PathStep;

use super::Value;
use super::lex::{Kind, Lexer, Rules};
use super::path::path_at;
use crate::diagnostic::{
    Diagnostic, DiagnosticCode, JsonPath, JsonPathSegment, Span, code_for_deserialize_error_kind,
};

/// A slice of a document's text with its file offset, so a nested parse reports file offsets.
/// Internal: the versioning machinery names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct JsonSlice<'a> {
    /// The text.
    pub text: &'a str,
    /// The byte offset of `text` in the file.
    pub offset: usize,
}

impl<'a> JsonSlice<'a> {
    /// A whole file's text, at offset 0.
    pub fn new(text: &'a str) -> Self {
        Self { text, offset: 0 }
    }

    /// Whether the text, after whitespace and comments, opens an object: the object form of a
    /// component that also reads a bare value.
    pub fn opens_an_object(&self) -> bool {
        self.first_byte() == Some(b'{')
    }

    /// The first byte of the text that is neither white space nor inside a comment: what
    /// kind of value it holds, by a look at its start. White space is `str::trim_start`'s
    /// (Unicode's, wider than JSON's: a peek, not a read); a `/*` comment without an end runs
    /// to the end of the text. `None` for a text of white space and comments only.
    pub fn first_byte(&self) -> Option<u8> {
        let mut rest = self.text;
        loop {
            rest = rest.trim_start();
            if let Some(after) = rest.strip_prefix("//") {
                rest = after.split_once('\n').map_or("", |(_, tail)| tail);
            } else if let Some(after) = rest.strip_prefix("/*") {
                rest = after.split_once("*/").map_or("", |(_, tail)| tail);
            } else {
                return rest.bytes().next();
            }
        }
    }

    /// The slice a `RawJson` captured from this slice's text, with its file offset: the pointer
    /// difference between the two. `None` when `raw` does not borrow from this text (an owned
    /// `RawJson`, or one taken from another document).
    pub fn nested(&self, raw: &RawJson<'a>) -> Option<JsonSlice<'a>> {
        let std::borrow::Cow::Borrowed(text) = raw.0 else {
            return None;
        };
        let start = (text.as_ptr() as usize).checked_sub(self.text.as_ptr() as usize)?;
        let end = start.checked_add(text.len())?;
        (end <= self.text.len()).then_some(JsonSlice {
            text,
            offset: self.offset + start,
        })
    }
}

/// A typed parse of a [`JsonSlice`] failed: facet's error, its span rebased to the file, and the
/// JSON members from the slice root to the failing value.
#[derive(Debug, thiserror::Error)]
#[error("the document slice did not deserialize")]
pub struct SliceError {
    /// Where in the file: facet's span plus the slice offset; the whole slice when facet gives
    /// no span (integer range errors).
    pub span: Span,
    /// The members from the slice root to the failing value, by their JSON names (a missing
    /// field included). The caller prefixes its own breadcrumb (the component key).
    pub path: JsonPath,
    /// facet's error; its `kind` picks the diagnostic code (see `code_for_deserialize_error_kind`).
    #[source]
    pub error: Box<DeserializeError>,
    /// The slice was empty or whitespace only: an end of input is then an empty document.
    blank: bool,
}

impl SliceError {
    /// The members from the slice root to the failing value: [`path`](Self::path).
    pub fn relative_path(&self) -> &JsonPath {
        &self.path
    }

    /// Whether the failure is at a member, not at the slice root.
    pub fn has_path(&self) -> bool {
        !self.path.is_empty()
    }

    /// Whether the slice was empty or whitespace only.
    pub fn is_blank(&self) -> bool {
        self.blank
    }

    /// facet's words for the failure, as its error kind renders them.
    pub fn kind_text(&self) -> String {
        self.error.kind.to_string()
    }

    /// Whether facet's words for the failure contain `words`: the question a family asks
    /// when facet has no kind of its own for a failure (jigsaw's unknown tag).
    pub fn describes(&self, words: &str) -> bool {
        self.kind_text().contains(words)
    }

    /// The table's code for the failure graded as at a member (`has_path`) or at the slice
    /// root: [`into_diagnostic`](Self::into_diagnostic) gives
    /// `code_with_path(self.has_path())`, a family that grades otherwise asks for the other.
    pub fn code_with_path(&self, has_path: bool) -> DiagnosticCode {
        code_for_deserialize_error_kind(&self.error.kind, has_path, self.blank)
    }

    /// The finding of the failed parse: an error told in [`Phase::Parse`] at `base` followed by
    /// the failing member, at the error's span, its code chosen by facet's error kind
    /// (`code_for_deserialize_error_kind`). `remap` gets that code and the error itself, and
    /// returns the code to tell: a family whose game reports a failure otherwise says so there,
    /// asking the error what it needs ([`relative_path`](Self::relative_path),
    /// [`describes`](Self::describes), or [`code_with_path`](Self::code_with_path) to grade it
    /// as at a member or at the root); the identity keeps the table's code. The builder changes
    /// the rest ([`Diagnostic::severity`], [`Diagnostic::phase`]).
    pub fn into_diagnostic(
        self,
        base: &JsonPath,
        remap: impl FnOnce(DiagnosticCode, &SliceError) -> DiagnosticCode,
    ) -> Diagnostic {
        let code = remap(self.code_with_path(self.has_path()), &self);
        let mut at = base.clone();
        at.append(&self.path);
        Diagnostic::error(code).at(at).span(self.span)
    }
}

/// Deserialises `T` from the slice's text with the jsonc entry point, the way every typed pack
/// file is read; a failure is reported at file offsets. An array where
/// `T` has a map is `InvalidType` at that level (facet alone reads `[]` as an empty map).
/// A JSON `null` for an `Option` member reads as `None`, as an absent member does
/// (jsoncpp's `isNull`); a family whose vanilla reader tells the two apart checks the JSON
/// itself.
///
/// Needs no stack of the caller that grows with the text: a slice nested to `MAX_JSON_DEPTH`
/// is read, and a deeper one refused with an error, on a thread with a megabyte of stack,
/// whatever `T` is (one with an untyped member, or one that holds itself, included).
/// `MAX_JSON_DEPTH` names what is outside that rule: WebAssembly, a target without a stack
/// switch, a host that cannot map the stack (a panic).
///
/// # Errors
///
/// Returns `SliceError` when `slice` does not read as `T`, with the file span and JSON path of the fault.
pub fn parse_at<'a, T: Facet<'a>>(slice: JsonSlice<'a>) -> Result<T, SliceError> {
    parse_in(slice).map_err(|failure| {
        let (span, members, error) = *failure;
        SliceError {
            span,
            path: members,
            error: Box::new(error),
            blank: slice.text.trim().is_empty(),
        }
    })
}

/// [`parse_at`] reading the object members `absent` names as not written when their value is
/// `null` (rule N1, `json::null`); a failure is reported against the text as written.
pub(crate) fn parse_without<'a, T: Facet<'a>>(
    slice: JsonSlice<'a>,
    absent: &[JsonPath],
) -> Result<T, SliceError> {
    parse_planned(slice, &super::read::Plan::absent(absent))
}

/// [`parse_at`] reading `slice` as `plan` says (members read as not written, scalars read as
/// other tokens); a failure is reported against the text as written.
pub(crate) fn parse_planned<'a, T: Facet<'a>>(
    slice: JsonSlice<'a>,
    plan: &super::read::Plan,
) -> Result<T, SliceError> {
    parse_read(slice, plan).map_err(|failure| {
        let (span, members, error) = *failure;
        SliceError {
            span,
            path: members,
            error: Box::new(error),
            blank: slice.text.trim().is_empty(),
        }
    })
}

/// [`parse_at`] with the member path as this module builds it.
pub(crate) fn parse_in<'a, T: Facet<'a>>(
    slice: JsonSlice<'a>,
) -> Result<T, Box<(Span, JsonPath, DeserializeError)>> {
    parse_read(slice, &super::read::Plan::default())
}

/// [`parse_in`], read as `plan` says.
fn parse_read<'a, T: Facet<'a>>(
    slice: JsonSlice<'a>,
    plan: &super::read::Plan,
) -> Result<T, Box<(Span, JsonPath, DeserializeError)>> {
    let read = super::read::from_str_borrowed_jsonc_planned::<T>(slice.text, plan);
    let value = read.map_err(|error| {
        let members = members_of(&error, slice.text);
        // The span follows the members: where they are not the place facet's span lies in
        // (an array element facet's span missed), the value they name.
        let at_span = error
            .span
            .map(|span| path_at(slice.text, span.offset as usize));
        let span = match at_span {
            Some(at) if at != members => {
                locate(slice, &members).unwrap_or_else(|| rebase(slice, error.span))
            }
            // A span of no length marks no text (a strict wrapper's refusal comes
            // without the token's extent): the value the members name.
            _ if error.span.is_some_and(|span| span.len == 0) => {
                locate(slice, &members).unwrap_or_else(|| rebase(slice, error.span))
            }
            _ => rebase(slice, error.span),
        };
        Box::new((span, members, error))
    })?;
    match array_for_a_map(T::SHAPE, slice) {
        Some(failure) => Err(failure),
        None => Ok(value),
    }
}

/// facet reads `[]` as an empty map; jsoncpp's object reads refuse an array. The first place
/// where `shape` has a map and the text an array is a type error there. Only a text
/// with an empty array in it is looked at, and it is read a second time, as a value, only
/// where `shape` has a map below its root: a shape without a map has no such place, and a map
/// that holds none (raw members, numbers) has it at the root alone, which the first byte of
/// the text tells.
fn array_for_a_map(
    shape: &'static Shape,
    slice: JsonSlice<'_>,
) -> Option<Box<(Span, JsonPath, DeserializeError)>> {
    if !has_empty_array(slice.text) {
        return None;
    }
    let mut members = JsonPath::new();
    let map = match map_at_the_root(shape, slice.text) {
        AtTheRoot::NoMap => return None,
        AtTheRoot::Array(map) => map,
        AtTheRoot::Unknown => {
            let json: Value = crate::json::from_str_jsonc(slice.text).ok()?;
            map_given_an_array(shape, &json, &mut members)?
        }
    };
    let span = locate(slice, &members).unwrap_or_else(|| rebase(slice, None));
    let error = DeserializeError {
        span: None,
        path: None,
        kind: facet_format::DeserializeErrorKind::TypeMismatch {
            expected: map,
            got: "array".into(),
        },
    };
    Some(Box::new((span, members, error)))
}

/// What the shape and the first byte of a text say of [`map_given_an_array`]'s answer.
enum AtTheRoot {
    /// No array stands for a map: the shape holds no map, or it is a map of values that hold
    /// none and the text is an object.
    NoMap,
    /// The text is an array and the shape this map, whose values hold none.
    Array(&'static Shape),
    /// The value has to be read: a map below the root, or a text that begins with a comment.
    Unknown,
}

/// What [`map_given_an_array`] finds for `shape`, where the shape and the first byte of
/// `text` (which a typed read has taken) say it without the text's value.
fn map_at_the_root(shape: &'static Shape, text: &str) -> AtTheRoot {
    let shape = see_through(shape);
    let Def::Map(map) = shape.def else {
        return if may_hold_a_map(shape, &mut Vec::new()) {
            AtTheRoot::Unknown
        } else {
            AtTheRoot::NoMap
        };
    };
    if is_kept_as_written(shape) {
        return AtTheRoot::NoMap;
    }
    if may_hold_a_map(map.v(), &mut Vec::new()) {
        return AtTheRoot::Unknown;
    }
    match text.trim_start().as_bytes().first() {
        Some(b'{') => AtTheRoot::NoMap,
        Some(b'[') => AtTheRoot::Array(shape),
        _ => AtTheRoot::Unknown,
    }
}

/// Whether [`map_given_an_array`] can find a map at `shape` or below it: the same steps, by
/// the shape alone. `seen` holds the shapes looked at, each once: one that holds itself, or
/// that several members hold, is not walked again (the first map found ends the search, so
/// a shape seen before has shown none).
fn may_hold_a_map(shape: &'static Shape, seen: &mut Vec<facet::ConstTypeId>) -> bool {
    let shape = see_through(shape);
    if is_kept_as_written(shape) || seen.contains(&shape.id) {
        return false;
    }
    if matches!(shape.def, Def::Map(_)) {
        return true;
    }
    seen.push(shape.id);
    element_of(shape).is_some_and(|element| may_hold_a_map(element, seen))
        || (matches!(shape.ty, Type::User(UserType::Struct(_)))
            && named_fields(shape, None).is_some_and(|fields| {
                fields
                    .iter()
                    .any(|field| may_hold_a_map(field.shape(), seen))
            }))
}

/// Whether [`map_given_an_array`] does not look into `shape`: a type read through a proxy,
/// and the two that keep the text as written.
fn is_kept_as_written(shape: &'static Shape) -> bool {
    shape.effective_proxy(None).is_some()
        || shape.id == Value::SHAPE.id
        || shape.id == <RawJson<'static> as Facet<'static>>::SHAPE.id
}

/// Whether `text` (which the jsonc reader has taken) may hold an empty array: `[` and `]` with
/// only white space and comments between them. Never `false` for a text that holds one; a
/// `[` and a `]` with white space alone between them count wherever they stand (in a string
/// too), which costs the second read and changes no answer.
///
/// The text is gone over once, whatever it holds. Each `[` is looked at with the white space
/// behind it alone, and the first that has a comment there hands the whole text to
/// [`has_array_of_comments`], which reads it as its tokens from the start: the end of a
/// comment is not looked for from every `[` that stands before it.
pub(super) fn has_empty_array(text: &str) -> bool {
    for (at, _) in text.match_indices('[') {
        match text[at + 1..].trim_start().as_bytes().first() {
            Some(b']') => return true,
            Some(b'/') => return has_array_of_comments(text),
            _ => {}
        }
    }
    false
}

/// Whether `text` holds an array of nothing but white space and comments (an array with
/// nothing at all between its brackets too), read as the jsonc reader reads it: a `[` or a
/// comment's marks in a string are text, a `[` in a comment is comment, a `//` comment ends
/// at its newline and a `/* */` comment at its first `*/`. One pass over the text.
pub(super) fn has_array_of_comments(text: &str) -> bool {
    let mut lexer = Lexer::new(text.as_bytes(), Rules::lenient(false));
    // Whether the last token was `[`, white space (`str::trim_start`'s, as the first look
    // takes it) and comments aside.
    let mut open = false;
    loop {
        if lexer.skip_trivia().is_some() {
            // A comment without an end closes no array.
            return false;
        }
        let Ok(Some(token)) = lexer.token() else {
            return false;
        };
        match token.kind {
            Kind::OpenArray => open = true,
            Kind::CloseArray if open => return true,
            Kind::Atom => open &= text[token.range()].chars().all(char::is_whitespace),
            _ => open = false,
        }
    }
}

/// The first map of `shape` that `json` gives an array, by the members to it.
fn map_given_an_array(
    shape: &'static Shape,
    json: &Value,
    members: &mut JsonPath,
) -> Option<&'static Shape> {
    let shape = see_through(shape);
    if is_kept_as_written(shape) {
        return None;
    }
    if let Def::Map(map) = shape.def {
        if json.as_array().is_some() {
            return Some(shape);
        }
        for (key, value) in json.as_object()? {
            members.push(key.as_str());
            let found = map_given_an_array(map.v(), value, members);
            if found.is_some() {
                return found;
            }
            members.pop();
        }
        return None;
    }
    if let (Some(element), Some(array)) = (element_of(shape), json.as_array()) {
        for (index, value) in array.iter().enumerate() {
            members.push_index(index);
            let found = map_given_an_array(element, value, members);
            if found.is_some() {
                return found;
            }
            members.pop();
        }
        return None;
    }
    if let (Type::User(UserType::Struct(_)), Some(object)) = (shape.ty, json.as_object()) {
        for field in named_fields(shape, None)? {
            let Some(value) = object.get(field.effective_name()) else {
                continue;
            };
            members.push(field.effective_name());
            let found = map_given_an_array(field.shape(), value, members);
            if found.is_some() {
                return found;
            }
            members.pop();
        }
    }
    None
}

/// The span, at file offsets, of the value `path` names below the root of `slice`; `None`
/// when a step of the path is not in the text. For a key written twice it is the last value,
/// the one the readers keep. The way a reader that works on a parsed value gives its
/// finding the span of the text it is about.
///
/// Each step reads one level of the text, so one call costs the levels on the path: for the
/// spans of many values of one document, walk the text once instead
/// ([`clamp_non_finite`](super::clamp_non_finite) does).
pub fn span_at(slice: JsonSlice<'_>, path: &JsonPath) -> Option<Span> {
    locate(slice, path)
}

/// The span of the value `members` names in `slice`; `None` when a step is not found.
fn locate(slice: JsonSlice<'_>, members: &JsonPath) -> Option<Span> {
    let mut current = slice;
    for step in members.segments() {
        current = enter(current, step)?;
    }
    Some(rebase(current, None))
}

/// The span of the value at `path` below `slice`, the last value of each member on the way
/// (the one that stands); when a step cannot be taken (a member that is not there, a key
/// the path does not spell unambiguously, a value of another kind), the span of the deepest
/// value reached. [`span_at`] for a finding that is to have a place whatever its path says.
pub fn span_reached(slice: JsonSlice<'_>, path: &JsonPath) -> Span {
    let mut current = slice;
    for segment in path.segments() {
        match enter(current, segment) {
            Some(next) => current = next,
            None => break,
        }
    }
    rebase(current, None)
}

/// The value one step below `slice`: its member `key` (the last of that key), or its element
/// `index`. One read of the level.
fn enter<'a>(slice: JsonSlice<'a>, segment: JsonPathSegment<'_>) -> Option<JsonSlice<'a>> {
    use indexmap::IndexMap;
    match segment {
        JsonPathSegment::Key(key) => {
            crate::json::from_str_borrowed_jsonc::<IndexMap<String, RawJson<'_>>>(slice.text)
                .ok()
                .and_then(|object| slice.nested(object.get(key)?))
        }
        JsonPathSegment::Index(index) => {
            crate::json::from_str_borrowed_jsonc::<Vec<RawJson<'_>>>(slice.text)
                .ok()
                .and_then(|array| slice.nested(array.get(index)?))
        }
    }
}

/// facet's span moved by the slice offset; the whole slice when there is none.
fn rebase(slice: JsonSlice<'_>, span: Option<facet::Span>) -> Span {
    let to_u32 = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    match span {
        Some(span) => Span {
            offset: to_u32(slice.offset).saturating_add(span.offset),
            len: span.len,
        },
        None => Span {
            offset: to_u32(slice.offset),
            len: to_u32(slice.text.len()),
        },
    }
}

/// The JSON members from the slice root to the failing value.
///
/// facet's span is exact (a member's key, or the offending token), so the members are read from
/// the text at the span: keys as written, map keys included. A missing field is the exception:
/// facet puts its span on an outer member, so its members come from facet's `path` (struct
/// fields by `effective_name`, so renamed fields read as written) followed by the field's JSON
/// name.
fn members_of(error: &DeserializeError, text: &str) -> JsonPath {
    let at_span = error.span.map(|span| path_at(text, span.offset as usize));
    if let facet_format::DeserializeErrorKind::MissingField {
        field,
        container_shape,
    } = error.kind
    {
        let (mut members, end) = error
            .path
            .as_ref()
            .map(|path| walk_path(path, text, at_span.as_ref()))
            .unwrap_or_default();
        // A walk that did not reach the field's struct (a map entry it cannot name) names the
        // members it is sure of, without the field.
        if end.is_some_and(|end| see_through(end).id == container_shape.id) {
            let name = named_fields(container_shape, None)
                .and_then(|fields| fields.iter().find(|f| f.name == field))
                .map_or(field, facet::Field::effective_name);
            members.push(name);
        }
        return members;
    }
    // A conversion failure of an array element (a strict scalar's proxy) can carry a span at
    // another element than the one facet's path names: where the path walks to its end and
    // agrees with the span's members up to array indices, its indices are the right ones.
    if let (Some(path), Some(span_members)) = (error.path.as_ref(), at_span.as_ref()) {
        let (members, end) = walk_path(path, text, Some(span_members));
        // Only for a scalar (or a proxied value): the failure of an enum or struct element is
        // where its span says (a tagged enum's unknown tag).
        let leaf = end.is_some_and(|shape| {
            let shape = see_through(shape);
            shape.effective_proxy(None).is_some() || matches!(shape.def, Def::Scalar)
        });
        if leaf && members.refines(span_members) {
            return members;
        }
    }
    at_span
        .or_else(|| {
            let path = error.path.as_ref()?;
            Some(walk_path(path, text, None).0)
        })
        .unwrap_or_default()
}

/// The members facet's `path` names. facet does not record every step (an `Option` around a
/// struct, a map entry), so the walk sees through options, pointers and transparent wrappers,
/// and takes a map entry it did not record from `hint` (the members at the span) when the
/// members so far agree with it. The shape is where the walk ended, `None` when it stopped early.
fn walk_path(
    path: &facet_path::Path,
    text: &str,
    hint: Option<&JsonPath>,
) -> (JsonPath, Option<&'static Shape>) {
    let mut members = JsonPath::new();
    let root: Option<Value> = crate::json::from_str_borrowed_jsonc(text).ok();
    let mut node = root.as_ref();
    let mut shape = path.shape;
    let mut variant: Option<usize> = None;
    for step in path.steps() {
        if variant.is_none() {
            shape = see_through(shape);
            if let Def::Map(map) = shape.def
                && !matches!(step, PathStep::MapKey(_) | PathStep::MapValue(_))
            {
                // An entry facet did not record.
                let Some(key) = hint.and_then(|hint| hint.key_after(&members)) else {
                    return (members, None);
                };
                node = node.and_then(|n| n.as_object()).and_then(|o| o.get(key));
                members.push(key);
                shape = see_through(map.v());
            }
        }
        match *step {
            PathStep::Field(index) => {
                let Some(field) = field_of(shape, variant, index as usize) else {
                    return (members, None);
                };
                // A flattened field's members are its parent's: no JSON level.
                if variant.is_none() && field.is_flattened() {
                    shape = field.shape();
                    continue;
                }
                // The one field of a newtype variant is the variant's value itself
                // (`Box(BoundingBox)` of an untagged enum reads the object, `{"Tag": value}`
                // of an externally tagged one has its key from the variant): no JSON level.
                if !variant.is_some_and(|variant| is_newtype_variant(shape, variant)) {
                    let name = field.effective_name();
                    members.push(name);
                    node = node.and_then(|n| n.as_object()).and_then(|o| o.get(name));
                }
                shape = field.shape();
                variant = None;
            }
            PathStep::Index(index) => {
                let Some(element) = element_of(shape) else {
                    return (members, None);
                };
                members.push_index(index as usize);
                node = node
                    .and_then(|n| n.as_array())
                    .and_then(|a| a.get(index as usize));
                shape = element;
            }
            PathStep::Variant(index) => {
                variant = Some(index as usize);
                if let Some(name) = external_tag(shape, index as usize) {
                    members.push(name);
                    node = node.and_then(|n| n.as_object()).and_then(|o| o.get(name));
                }
            }
            PathStep::MapValue(entry) => {
                let (Def::Map(map), Some((key, value))) = (
                    shape.def,
                    node.and_then(|n| n.as_object())
                        .and_then(|o| o.iter().nth(entry as usize)),
                ) else {
                    return (members, None);
                };
                members.push(key.as_str());
                node = Some(value);
                shape = map.v();
            }
            PathStep::OptionSome | PathStep::Deref | PathStep::Inner => {}
            PathStep::Proxy => match shape.effective_proxy(None) {
                Some(proxy) => shape = proxy.shape,
                None => return (members, None),
            },
            _ => return (members, None),
        }
    }
    (members, Some(shape))
}

/// `shape` without the wrappers that add no JSON level: `Option`, pointers, transparent types.
fn see_through(mut shape: &'static Shape) -> &'static Shape {
    loop {
        shape = match shape.def {
            Def::Option(option) => option.t(),
            Def::Pointer(def) => match def.pointee() {
                Some(pointee) => pointee,
                None => return shape,
            },
            _ => match shape.inner {
                Some(inner) if shape.is_transparent() => inner,
                _ => return shape,
            },
        };
    }
}

/// The fields of a struct shape, or of variant `variant` of an enum shape.
pub(crate) fn named_fields(
    shape: &'static Shape,
    variant: Option<usize>,
) -> Option<&'static [facet::Field]> {
    match shape.ty {
        Type::User(UserType::Struct(st)) => Some(st.fields),
        Type::User(UserType::Enum(en)) => Some(en.variants.get(variant?)?.data.fields),
        _ => None,
    }
}

fn field_of(
    shape: &'static Shape,
    variant: Option<usize>,
    index: usize,
) -> Option<&'static facet::Field> {
    named_fields(shape, variant)?.get(index)
}

/// Whether variant `variant` of the enum `shape` is a tuple variant of one field.
fn is_newtype_variant(shape: &'static Shape, variant: usize) -> bool {
    let Type::User(UserType::Enum(en)) = shape.ty else {
        return false;
    };
    en.variants.get(variant).is_some_and(|variant| {
        variant.data.fields.len() == 1
            && matches!(
                variant.data.kind,
                facet::StructKind::TupleStruct | facet::StructKind::Tuple
            )
    })
}

/// The element shape of a list, array or slice.
pub(crate) fn element_of(shape: &'static Shape) -> Option<&'static Shape> {
    match shape.def {
        Def::List(list) => Some(list.t()),
        Def::Array(array) => Some(array.t()),
        Def::Slice(slice) => Some(slice.t()),
        _ => None,
    }
}

/// The JSON key of variant `index` of an externally tagged enum (`{"Variant": …}`); `None` for
/// untagged and internally tagged enums, whose variants add no key.
fn external_tag(shape: &'static Shape, index: usize) -> Option<&'static str> {
    if shape.is_untagged() || shape.get_tag_attr().is_some() {
        return None;
    }
    match shape.ty {
        Type::User(UserType::Enum(en)) => {
            en.variants.get(index).map(facet::Variant::effective_name)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use super::*;
    use crate::diagnostic::codes as core_codes;

    #[test]
    fn an_object_is_recognised_after_comments_and_whitespace() {
        let opens = |text| JsonSlice::new(text).opens_an_object();
        assert!(opens(" {\"value\": 1}"));
        assert!(opens("// note\n {}"));
        assert!(opens("/* x */{}"));
        assert!(!opens(" 16"));
        assert!(!opens("\"text\""));
    }

    /// Today's recipe at every call site, verbatim: the oracle of `into_diagnostic`.
    fn recipe(error: &SliceError, base: &JsonPath, slice: JsonSlice<'_>) -> Diagnostic {
        use crate::diagnostic::{Location, Phase, Severity};
        let blank = slice.text.trim().is_empty();
        let code =
            code_for_deserialize_error_kind(&error.error.kind, !error.path.is_empty(), blank);
        let mut at = base.clone();
        at.append(&error.path);
        Diagnostic::new(
            code,
            Severity::Error,
            Phase::Parse,
            Location::new(None, None, at, Some(error.span)),
        )
    }

    #[test]
    fn into_diagnostic_is_the_recipe_of_the_call_sites() {
        let base = JsonPath::of_keys(["minecraft:block", "components"]);
        for text in [
            r#"{"v": "x"}"#,
            r#"{"v": 300}"#,
            "{}",
            r#"{"v": 1,"#,
            "",
            "  ",
            "[1]",
        ] {
            let slice = JsonSlice::new(text);
            let error = parse_at::<Inner>(slice).unwrap_err();
            let expected = recipe(&error, &base, slice);
            assert_eq!(
                error.into_diagnostic(&base, |code, _| code),
                expected,
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_blank_slice_is_an_empty_document() {
        let error = parse_at::<Inner>(JsonSlice::new(" \n ")).unwrap_err();
        let finding = error.into_diagnostic(&JsonPath::new(), |code, _| code);
        assert!(finding.code.name() == "EmptyDocument", "{:?}", finding.code);
    }

    #[test]
    fn remap_gets_the_code_and_the_error() {
        let error = parse_at::<Inner>(JsonSlice::new(r#"{"v": "x"}"#)).unwrap_err();
        let text = error.error.kind.to_string();
        let finding = error.into_diagnostic(&JsonPath::of_keys(["base"]), |code, error| {
            assert!(code.name() == "InvalidType", "{code:?}");
            assert_eq!(*error.relative_path(), JsonPath::of_keys(["v"]));
            assert!(error.has_path());
            assert!(!error.is_blank());
            assert_eq!(error.kind_text(), text);
            assert!(error.describes(&text[..3]));
            assert!(!error.describes("no such words in it"));
            assert_eq!(error.code_with_path(error.has_path()), code);
            DiagnosticCode::from(&core_codes::INVALID_VALUE)
        });
        assert!(finding.is(&core_codes::INVALID_VALUE));
        assert_eq!(finding.location.json_path, JsonPath::of_keys(["base", "v"]));
    }

    /// The call sites that grade the failure as if it were (or were not) at a member: an item
    /// scalar (never), a catalog entry (whenever its full path is not the root).
    #[test]
    fn remap_can_grade_the_failure_with_another_has_path() {
        let text = r#"{"v": [1]}"#;
        let error = parse_at::<Inner>(JsonSlice::new(text)).unwrap_err();
        assert!(
            error.code_with_path(true).name() == "InvalidType",
            "at a member, a token of the wrong kind is a type error"
        );
        let finding = error.into_diagnostic(&JsonPath::of_keys(["base"]), |_, error| {
            error.code_with_path(false)
        });
        assert!(
            finding.code.name() == "MalformedInput",
            "graded as at the root, it is malformed input: {:?}",
            finding.code
        );
        assert_eq!(finding.location.json_path, JsonPath::of_keys(["base", "v"]));
    }

    #[test]
    fn slice_error_chains_the_facet_error_as_its_source() {
        use std::error::Error;
        let error = parse_at::<Inner>(JsonSlice::new(r#"{"v": "x"}"#)).unwrap_err();
        assert!(error.source().is_some(), "the facet error is the source");
    }

    #[derive(Debug, Facet)]
    struct Inner {
        v: u8,
    }

    #[derive(Debug, Facet)]
    struct Renamed {
        #[facet(rename = "minecraft:value")]
        value: u8,
    }

    #[derive(Debug, Facet)]
    struct Outer {
        #[facet(default)]
        list: Vec<Inner>,
        #[facet(default)]
        map: IndexMap<String, Inner>,
        #[facet(default)]
        option: Option<Inner>,
        #[facet(default)]
        boxed: Option<Box<Inner>>,
        #[facet(default)]
        renamed: Option<Renamed>,
        #[facet(default)]
        tagged: Option<Tagged>,
    }

    #[derive(Debug, Facet)]
    #[repr(u8)]
    #[allow(dead_code)]
    enum Tagged {
        First { v: u8 },
        Second { w: u8 },
    }

    /// The member path `parse_at` reports for `text` parsed as `Outer`, joined by `/`.
    fn failing_path(text: &str) -> String {
        match parse_in::<Outer>(JsonSlice::new(text)) {
            Ok(_) => panic!("{text} parsed"),
            Err(error) => error.1.to_string(),
        }
    }

    #[test]
    fn a_list_element_is_named_by_its_index() {
        assert_eq!(
            failing_path(r#"{ "list": [{ "v": 1 }, { "v": "x" }] }"#),
            "list/1/v"
        );
    }

    #[test]
    fn a_map_entry_is_named_by_its_key() {
        assert_eq!(
            failing_path(r#"{ "map": { "a": { "v": 1 }, "b": { "v": true } } }"#),
            "map/b/v"
        );
    }

    #[test]
    fn options_and_boxes_add_no_member() {
        assert_eq!(failing_path(r#"{ "option": { "v": [] } }"#), "option/v");
        assert_eq!(failing_path(r#"{ "boxed": { "v": [] } }"#), "boxed/v");
    }

    #[test]
    fn a_renamed_field_is_named_as_written() {
        assert_eq!(
            failing_path(r#"{ "renamed": { "minecraft:value": "x" } }"#),
            "renamed/minecraft:value"
        );
    }

    #[test]
    fn a_missing_field_is_named_by_its_json_name() {
        assert_eq!(
            failing_path(r#"{ "renamed": {} }"#),
            "renamed/minecraft:value"
        );
        assert_eq!(failing_path(r#"{ "list": [{}] }"#), "list/0/v");
    }

    #[test]
    fn a_missing_field_in_a_map_entry_is_named_through_the_entry_key() {
        // facet records neither the entry nor a span inside it, so the path stops at the map
        // rather than name the field under the wrong member.
        assert_eq!(
            failing_path(r#"{ "map": { "a": { "v": 1 }, "b": {} } }"#),
            "map"
        );
    }

    #[test]
    fn an_externally_tagged_variant_is_a_member() {
        assert_eq!(
            failing_path(r#"{ "tagged": { "Second": { "w": "x" } } }"#),
            "tagged/Second/w"
        );
    }

    #[test]
    fn a_syntax_error_is_placed_at_its_token() {
        assert_eq!(failing_path(r#"{ "list": [ }"#), "list/0");
    }
}
