//! The writer: typed values to JSON text.
//!
//! facet-format walks the value (declaration order, renames, proxies, `skip_serializing`); this
//! module is the text backend. It adds what facet-json's own writer lacks: a document's unknown
//! members (its `Unknown` table) after the known members of each object, by the object's path
//! (`merge_unknown`), `f32` in the shortest form that
//! reads back to the same `f32` with a `.0` on integral values (jsoncpp's real tag survives), a
//! `Value`'s numbers with their int/real tag, no `null` for an absent `Option`, and
//! `EncodeError::NotRepresentable` instead of `null` for a non-finite number.
#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "this module is the JSON door: the one caller of facet-json"
)]

use std::borrow::Cow;

use facet::{Facet, Peek, ScalarType, Shape, Type, UserType};
use facet_format::{FormatSerializer, ScalarValue, SerializeError};

use super::path::MemberPath;
use super::stack::{is_low, with_stack_for};
use super::text::MAX_JSON_DEPTH;
use super::{EncodeError, Indent, Unrepresentable, Value, WriteStyle};
use crate::diagnostic::{
    Diagnostic, DiagnosticCode, JsonPath, JsonPathSegment, Location, Phase, Severity, VanillaText,
};
use crate::document::Unknown;

/// Writes `value` as JSON text to `out` in `style`. On failure nothing is appended to `out`.
///
/// A value whose text would nest deeper than `MAX_JSON_DEPTH` is refused
/// ([`Unrepresentable::TooDeep`]): no reader of the crate reads such a text. Needs no stack
/// of the caller that grows with the value: one nested to the limit is written on a thread
/// with a megabyte of stack (where little stack is left the walk is made again on a stack
/// of its own, on this thread). `MAX_JSON_DEPTH` names what is outside that rule:
/// WebAssembly, a target without a stack switch, a host that cannot map the stack (a
/// panic).
///
/// # Errors
///
/// Returns `EncodeError::NotRepresentable` when the value has no JSON form: a non-finite number, an unsupported shape, or nesting past `MAX_JSON_DEPTH`.
pub fn write_json<'f, T: Facet<'f>>(
    value: &T,
    out: &mut Vec<u8>,
    style: WriteStyle,
) -> Result<(), EncodeError> {
    write_in(value, out, style).map_err(|(at, what)| EncodeError::NotRepresentable {
        at: at.to_json_path(),
        what,
    })
}

/// [`write_json`], with the unknown members of `unknown` (a document's table, its paths from
/// the root of `value`) after the known members of the object at each path, as
/// [`merge_unknown`] puts them: the value is written, read as a `Value`, given the members and
/// written again (only when the table is not empty).
///
/// Returns the findings of the write: a member whose object `value` does not hold is not
/// written and is told as a `DroppedField` warning at the member's path.
///
/// # Errors
///
/// As [`write_json`].
pub fn write_json_with<'f, T: Facet<'f>>(
    value: &T,
    unknown: &Unknown,
    out: &mut Vec<u8>,
    style: WriteStyle,
) -> Result<Vec<Diagnostic>, EncodeError> {
    write_json_with_precedence(value, unknown, Precedence::UnknownWins, out, style)
}

/// [`write_json_with`], with `precedence` deciding a key the object already holds
/// ([`merge_unknown_with`]).
///
/// # Errors
///
/// As [`write_json`].
pub fn write_json_with_precedence<'f, T: Facet<'f>>(
    value: &T,
    unknown: &Unknown,
    precedence: Precedence,
    out: &mut Vec<u8>,
    style: WriteStyle,
) -> Result<Vec<Diagnostic>, EncodeError> {
    if unknown.is_empty() {
        return write_json(value, out, style).map(|()| Vec::new());
    }
    let mut text = Vec::new();
    write_json(value, &mut text, WriteStyle::default())?;
    // The writer's output is JSON this reader accepts; were it not, the value has no JSON form.
    let mut root = std::str::from_utf8(&text)
        .ok()
        .and_then(|text| super::from_str::<Value>(text).ok())
        .ok_or(EncodeError::NotRepresentable {
            at: JsonPath::new(),
            what: Unrepresentable::Unsupported,
        })?;
    let told = merge_unknown_with(&mut root, unknown, precedence);
    write_json(&root, out, style)?;
    Ok(told)
}

/// Puts the unknown members of `unknown` into `root`, each object's after its members in
/// document order, by the paths of their objects from `root`: what a written document has of
/// them ([`write_json_with`]), and what a writer that looks at the whole document before it
/// writes it uses (block decides its format by the members the document has).
///
/// A member of a key the object already holds replaces that member's value in place, without
/// a word: the member as the file had it, which the read kept beside a typed default (item's
/// `start_using` of a format that does not bind it); the text the writer wrote before the
/// table had both, and its reader took the second. Returns the findings: a member whose object
/// `root` does not hold (an edit removed or moved it) is left out and told as a
/// `DroppedField` warning of the write at the member's path.
#[must_use]
pub fn merge_unknown(root: &mut Value, unknown: &Unknown) -> Vec<Diagnostic> {
    merge_unknown_with(root, unknown, Precedence::UnknownWins)
}

/// Which value a written object keeps where a document's unknown member has the key of a
/// member the writer wrote ([`merge_unknown_with`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Precedence {
    /// The unknown member's value replaces the written one in place, untold: the member as the
    /// file had it, which the read kept beside a typed default ([`merge_unknown`]).
    #[default]
    UnknownWins,
    /// The written member stays and the unknown one is left out, untold: a writer that writes
    /// every member it knows from the value (dialogue, behavior trees), so a member of the
    /// same key in the table is a stale copy of it.
    KnownWins,
}

/// [`merge_unknown`] with `precedence` deciding a key the object already holds; a member whose
/// object `root` does not hold is told as there under either.
#[must_use]
pub fn merge_unknown_with(
    root: &mut Value,
    unknown: &Unknown,
    precedence: Precedence,
) -> Vec<Diagnostic> {
    let mut told = Vec::new();
    for owner in 0..unknown.owner_count() {
        let path = unknown.owner_path(owner);
        let mut object = Some(&mut *root);
        for segment in path.segments() {
            object = object.and_then(|value| match segment {
                JsonPathSegment::Key(key) => value.as_object_mut()?.get_mut(key),
                JsonPathSegment::Index(index) => value.as_array_mut()?.get_mut(index),
            });
        }
        let object = object.and_then(Value::as_object_mut);
        match object {
            Some(object) => {
                for (key, value) in unknown.owner_members(owner) {
                    if precedence == Precedence::KnownWins && object.contains_key(key) {
                        continue;
                    }
                    object.insert(&**key, value.clone());
                }
            }
            None => {
                for (key, _) in unknown.owner_members(owner) {
                    told.push(dropped(path.child(key)));
                }
            }
        }
    }
    told
}

/// [`write_json`] with the failure's member path as this module builds it.
pub(crate) fn write_in<'f, T: Facet<'f>>(
    value: &T,
    out: &mut Vec<u8>,
    style: WriteStyle,
) -> Result<(), (MemberPath, Unrepresentable)> {
    // facet walks the value by a call of itself for each level. The walk starts on the stack
    // there is and gives up where little of it is left; it is then made again from the start
    // on a stack that holds every depth a text may have.
    match write_once(value, out, style, true) {
        Written::OutOfStack => with_stack_for(MAX_JSON_DEPTH, || {
            match write_once(value, out, style, false) {
                Written::Whole => Ok(()),
                Written::Refused(at, what) => Err((at, what)),
                // Only a wary walk gives up for the stack, and this one is not: were it to,
                // nothing was written, and the caller is told so.
                Written::OutOfStack => Err((MemberPath::default(), Unrepresentable::Unsupported)),
            }
        }),
        Written::Whole => Ok(()),
        Written::Refused(at, what) => Err((at, what)),
    }
}

/// How one walk of a value ended.
enum Written {
    /// The text is in `out`.
    Whole,
    /// The value has no JSON spelling at the member named; nothing was appended.
    Refused(MemberPath, Unrepresentable),
    /// The walk gave up for the stack left; nothing was appended.
    OutOfStack,
}

/// One walk of `value` into `out`. A `wary` walk gives up where the stack runs low.
fn write_once<'f, T: Facet<'f>>(
    value: &T,
    out: &mut Vec<u8>,
    style: WriteStyle,
    wary: bool,
) -> Written {
    let start = out.len();
    let mut writer = Writer {
        out,
        style,
        frames: Vec::new(),
        failure: None,
        wary,
        out_of_stack: false,
    };
    if let Err(error) = facet_format::serialize_root(&mut writer, Peek::new(value)) {
        let what = match (&error, writer.failure) {
            (SerializeError::Backend(()), Some(what)) => what,
            _ => Unrepresentable::Unsupported,
        };
        let at = writer.path();
        writer.out.truncate(start);
        return if writer.out_of_stack {
            Written::OutOfStack
        } else {
            Written::Refused(at, what)
        };
    }
    if style.trailing_newline {
        writer.out.push(b'\n');
    }
    Written::Whole
}

/// The finding of an unknown member the writer leaves out, at its path.
fn dropped(at: JsonPath) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::DroppedField {
            text: Some(VanillaText::Fixed(UNKNOWN_MEMBER_NOT_WRITTEN)),
        },
        Severity::Warning,
        Phase::Write,
        Location::new(None, None, at, None),
    )
}

/// The text of the finding of an unknown member that is not written: its object is no longer
/// in the value.
pub const UNKNOWN_MEMBER_NOT_WRITTEN: &str =
    "an unknown member of the document is not written: the value no longer holds its object";

/// [`write_json`] into a string in the default style (two spaces, a trailing newline); the
/// same refusal of a value nested past the limit, and the same stack.
///
/// # Errors
///
/// Returns `EncodeError` under the same conditions as [`write_json`].
pub fn to_json<'f, T: Facet<'f>>(value: &T) -> Result<String, EncodeError> {
    let mut out = Vec::new();
    write_json(value, &mut out, WriteStyle::default())?;
    // The writer emits only UTF-8: keys and strings come from `str`, the rest is ASCII.
    Ok(String::from_utf8(out).unwrap_or_default())
}

/// `value` as facet-json writes a `Value`: on one line, an integral real as an integer
/// (`1.0` is `1`), which is how a conversion that types a `Value` by writing and reading it
/// wants it. The one call of facet-json's own writer in the crate family.
///
/// A value nested deeper than `MAX_JSON_DEPTH` is refused ([`Unrepresentable::TooDeep`], at
/// the array or object past the limit): no reader reads such a text. facet-json writes a
/// value by a call of itself for each level, so the depth is counted first and the write
/// run with the stack that depth can take, as a typed read is: one megabyte of stack on the
/// calling thread is enough at any depth.
///
/// # Errors
///
/// Returns `EncodeError::NotRepresentable` when `value` is nested past `MAX_JSON_DEPTH` or facet-json cannot write it.
pub fn to_facet_json(value: &Value) -> Result<String, EncodeError> {
    let levels = levels_of(value).map_err(|at| EncodeError::NotRepresentable {
        at: at.to_json_path(),
        what: Unrepresentable::TooDeep,
    })?;
    with_stack_for(levels, || facet_json::to_string(value)).map_err(|_| {
        EncodeError::NotRepresentable {
            at: crate::diagnostic::JsonPath::new(),
            what: Unrepresentable::Unsupported,
        }
    })
}

/// [`to_facet_json`], but for a real that is a negative zero, which is written `-0.0` and
/// not `0`: an integer has no sign, so the text of `to_facet_json` loses it. For a
/// conversion that types a `Value` into a member that holds a float; one that types it into
/// an integer wants [`to_facet_json`]'s `0`. Without a negative zero in `value` the two
/// texts are the same.
///
/// # Errors
///
/// Returns `EncodeError` under the same conditions as [`to_facet_json`].
pub fn to_facet_json_signed(value: &Value) -> Result<String, EncodeError> {
    fn negative_zero(value: &Value) -> bool {
        value.as_number().is_some_and(|number| {
            let real = number.to_f64_lossy();
            number.is_float() && real == 0.0 && real.is_sign_negative()
        })
    }
    fn holds_one(value: &Value) -> bool {
        if let Some(array) = value.as_array() {
            array.iter().any(holds_one)
        } else if let Some(object) = value.as_object() {
            object.iter().any(|(_, member)| holds_one(member))
        } else {
            negative_zero(value)
        }
    }
    // Arrays and objects are written here, one line and no space as facet-json writes
    // them; every scalar and key by facet-json, but the negative zero.
    fn write(value: &Value, out: &mut String) -> Result<(), ()> {
        if let Some(array) = value.as_array() {
            out.push('[');
            for (index, element) in array.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write(element, out)?;
            }
            out.push(']');
        } else if let Some(object) = value.as_object() {
            out.push('{');
            for (index, (key, member)) in object.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push_str(&facet_json::to_string(&Value::from(key.as_str())).map_err(|_| ())?);
                out.push(':');
                write(member, out)?;
            }
            out.push('}');
        } else if negative_zero(value) {
            out.push_str("-0.0");
        } else {
            out.push_str(&facet_json::to_string(value).map_err(|_| ())?);
        }
        Ok(())
    }
    // The depth is checked, and bounded, by the plain text first.
    let plain = to_facet_json(value)?;
    let levels = levels_of(value).map_err(|at| EncodeError::NotRepresentable {
        at: at.to_json_path(),
        what: Unrepresentable::TooDeep,
    })?;
    with_stack_for(levels, || {
        if !holds_one(value) {
            return Ok(plain);
        }
        let mut out = String::with_capacity(plain.len() + 8);
        write(value, &mut out).map(|()| out)
    })
    .map_err(|()| EncodeError::NotRepresentable {
        at: crate::diagnostic::JsonPath::new(),
        what: Unrepresentable::Unsupported,
    })
}

/// How deep `value` nests, counted without a call for each level; the path of the first
/// array or object past `MAX_JSON_DEPTH` when it has one.
// `pub(super)`: the unknown-member walk sizes its stack by it.
pub(super) fn levels_of(value: &Value) -> Result<usize, MemberPath> {
    /// The step into an element or a member.
    enum Step<'v> {
        Index(usize),
        Key(&'v str),
    }
    /// The elements or members of an open array or object still to visit.
    type Open<'v> = Box<dyn Iterator<Item = (Step<'v>, &'v Value)> + 'v>;
    fn open_of(value: &Value) -> Option<Open<'_>> {
        if let Some(array) = value.as_array() {
            let elements = array.iter().enumerate();
            return Some(Box::new(
                elements.map(|(index, element)| (Step::Index(index), element)),
            ));
        }
        let members = value.as_object()?.iter();
        Some(Box::new(
            members.map(|(key, member)| (Step::Key(key.as_str()), member)),
        ))
    }
    let mut at = MemberPath::default();
    let mut open: Vec<Open<'_>> = Vec::new();
    let mut deepest = 0;
    if let Some(root) = open_of(value) {
        open.push(root);
    }
    while let Some(innermost) = open.last_mut() {
        let Some((step, next)) = innermost.next() else {
            // The container is done, and with it the step that led into it.
            open.pop();
            if !open.is_empty() {
                at.pop();
            }
            continue;
        };
        match step {
            Step::Index(index) => at.push_index(index),
            Step::Key(key) => at.push_key(key),
        }
        match open_of(next) {
            Some(_) if open.len() >= MAX_JSON_DEPTH => return Err(at),
            Some(inner) => {
                open.push(inner);
                deepest = deepest.max(open.len());
            }
            None => at.pop(),
        }
    }
    Ok(deepest.max(open_of(value).map_or(0, |_| 1)))
}

/// One open object or array.
struct Frame {
    /// An object (`{}`) or an array (`[]`).
    object: bool,
    /// Members (elements) written so far.
    count: usize,
    /// The key of the member being written.
    key: Option<String>,
}

/// The text backend for facet-format's walk.
struct Writer<'o> {
    out: &'o mut Vec<u8>,
    style: WriteStyle,
    frames: Vec<Frame>,
    /// Why the walk stopped, when the writer stopped it.
    failure: Option<Unrepresentable>,
    /// Whether the walk gives up where the stack runs low.
    wary: bool,
    /// The walk gave up for the stack left.
    out_of_stack: bool,
}

impl Writer<'_> {
    /// Where the writer is: the open objects' current keys and the arrays' current indices.
    fn path(&self) -> MemberPath {
        let mut path = MemberPath::default();
        for frame in &self.frames {
            match (&frame.key, frame.object) {
                (Some(key), true) => path.push_key(key),
                (_, false) if frame.count > 0 => path.push_index(frame.count - 1),
                _ => {}
            }
        }
        path
    }

    fn newline(&mut self) {
        let depth = self.frames.len();
        match self.style.indent {
            Indent::Compact => {}
            Indent::Spaces(n) => {
                self.out.push(b'\n');
                self.out
                    .extend(std::iter::repeat_n(b' ', usize::from(n) * depth));
            }
            Indent::Tab => {
                self.out.push(b'\n');
                self.out.extend(std::iter::repeat_n(b'\t', depth));
            }
        }
    }

    /// Separator and indentation before an array element.
    fn before_value(&mut self) {
        if let Some(frame) = self.frames.last_mut()
            && !frame.object
        {
            frame.count += 1;
            if frame.count > 1 {
                self.out.push(b',');
            }
            self.newline();
        }
    }

    /// Opens an object or an array. One nested deeper than `MAX_JSON_DEPTH` is refused: no
    /// reader of the crate reads such a text, and the walk that leads here is a call for
    /// each level.
    fn open(&mut self, object: bool) -> Result<(), ()> {
        if self.frames.len() >= MAX_JSON_DEPTH {
            return self.fail(Unrepresentable::TooDeep);
        }
        if self.wary && is_low() {
            self.out_of_stack = true;
            return Err(());
        }
        self.before_value();
        self.out.push(if object { b'{' } else { b'[' });
        self.frames.push(Frame {
            object,
            count: 0,
            key: None,
        });
        Ok(())
    }

    fn close(&mut self) -> Result<(), ()> {
        let frame = self.frames.pop().ok_or(())?;
        if frame.count > 0 {
            self.newline();
        }
        self.out.push(if frame.object { b'}' } else { b']' });
        Ok(())
    }

    fn key(&mut self, key: &str) -> Result<(), ()> {
        let frame = self.frames.last_mut().filter(|f| f.object).ok_or(())?;
        frame.count += 1;
        frame.key = Some(key.to_owned());
        if frame.count > 1 {
            self.out.push(b',');
        }
        self.newline();
        self.string(key);
        self.out.push(b':');
        if self.style.indent != Indent::Compact {
            self.out.push(b' ');
        }
        Ok(())
    }

    fn string(&mut self, s: &str) {
        self.out.push(b'"');
        for c in s.chars() {
            match c {
                '"' => self.out.extend_from_slice(b"\\\""),
                '\\' => self.out.extend_from_slice(b"\\\\"),
                '\n' => self.out.extend_from_slice(b"\\n"),
                '\r' => self.out.extend_from_slice(b"\\r"),
                '\t' => self.out.extend_from_slice(b"\\t"),
                '\u{08}' => self.out.extend_from_slice(b"\\b"),
                '\u{0C}' => self.out.extend_from_slice(b"\\f"),
                c if c.is_control() && c.is_ascii() => {
                    self.out
                        .extend_from_slice(format!("\\u{:04x}", u32::from(c)).as_bytes());
                }
                c => {
                    let mut buf = [0; 4];
                    self.out
                        .extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
                }
            }
        }
        self.out.push(b'"');
    }

    fn raw(&mut self, text: &str) {
        self.before_value();
        self.out.extend_from_slice(text.as_bytes());
    }

    /// A float in the shortest form that reads back to the same value, `.0` on integral values;
    /// `Debug` of `f32` / `f64` is exactly that, with an exponent for very large or small values.
    fn float(&mut self, text: &str, x: f64) -> Result<(), ()> {
        if x.is_nan() {
            return self.fail(Unrepresentable::NaN);
        }
        if x.is_infinite() {
            return self.fail(if x > 0.0 {
                Unrepresentable::Infinity
            } else {
                Unrepresentable::NegativeInfinity
            });
        }
        self.raw(text);
        Ok(())
    }

    fn f32(&mut self, x: f32) -> Result<(), ()> {
        self.float(&format!("{x:?}"), f64::from(x))
    }

    fn f64(&mut self, x: f64) -> Result<(), ()> {
        self.float(&format!("{x:?}"), x)
    }

    fn fail(&mut self, what: Unrepresentable) -> Result<(), ()> {
        self.before_value();
        self.failure = Some(what);
        Err(())
    }

    /// A `Value`, written by this backend so a number keeps its int/real tag.
    fn value(&mut self, value: &Value) -> Result<(), ()> {
        if let Some(object) = value.as_object() {
            self.open(true)?;
            for (key, item) in object {
                self.key(key.as_str())?;
                self.value(item)?;
            }
            return self.close();
        }
        if let Some(array) = value.as_array() {
            self.open(false)?;
            for item in array {
                self.value(item)?;
            }
            return self.close();
        }
        if let Some(number) = value.as_number() {
            if number.is_float() {
                return self.f64(number.to_f64_lossy());
            }
            let text = match (number.to_i64(), number.to_u64()) {
                (Some(i), _) => i.to_string(),
                (None, Some(u)) => u.to_string(),
                (None, None) => return self.fail(Unrepresentable::Unsupported),
            };
            self.raw(&text);
            return Ok(());
        }
        if let Some(s) = value.as_string() {
            self.before_value();
            self.string(s.as_str());
            return Ok(());
        }
        match value.as_bool() {
            Some(true) => self.raw("true"),
            Some(false) => self.raw("false"),
            None if value.is_null() => self.raw("null"),
            None => return self.fail(Unrepresentable::Unsupported),
        }
        Ok(())
    }
}

impl FormatSerializer for Writer<'_> {
    type Error = ();

    fn begin_struct(&mut self) -> Result<(), ()> {
        self.open(true)
    }

    fn field_key(&mut self, key: &str) -> Result<(), ()> {
        self.key(key)
    }

    fn end_struct(&mut self) -> Result<(), ()> {
        self.close()
    }

    fn begin_seq(&mut self) -> Result<(), ()> {
        self.open(false)
    }

    fn end_seq(&mut self) -> Result<(), ()> {
        self.close()
    }

    fn scalar(&mut self, scalar: ScalarValue<'_>) -> Result<(), ()> {
        match scalar {
            ScalarValue::Null | ScalarValue::Unit => self.raw("null"),
            ScalarValue::Bool(b) => self.raw(if b { "true" } else { "false" }),
            ScalarValue::Char(c) => {
                self.before_value();
                self.string(c.encode_utf8(&mut [0; 4]));
            }
            ScalarValue::I64(i) => self.raw(&i.to_string()),
            ScalarValue::U64(u) => self.raw(&u.to_string()),
            ScalarValue::I128(i) => self.raw(&i.to_string()),
            ScalarValue::U128(u) => self.raw(&u.to_string()),
            ScalarValue::F64(x) => return self.f64(x),
            ScalarValue::Str(s) => {
                self.before_value();
                self.string(&s);
            }
            _ => return self.fail(Unrepresentable::Unsupported),
        }
        Ok(())
    }

    fn typed_scalar(&mut self, scalar_type: ScalarType, value: Peek<'_, '_>) -> Result<(), ()> {
        match scalar_type {
            ScalarType::F32 => self.f32(*value.get::<f32>().map_err(|_| ())?),
            ScalarType::F64 => self.f64(*value.get::<f64>().map_err(|_| ())?),
            _ => {
                // facet-format's normalisation, for every other scalar.
                let normalised = match scalar_type {
                    ScalarType::Bool => ScalarValue::Bool(*value.get::<bool>().map_err(|_| ())?),
                    ScalarType::Unit => ScalarValue::Null,
                    _ => match value.as_str() {
                        Some(s) => ScalarValue::Str(Cow::Borrowed(s)),
                        None => integer(scalar_type, value).ok_or(())?,
                    },
                };
                self.scalar(normalised)
            }
        }
    }

    fn should_omit_field(&self, _field: &facet::FieldItem, value: Peek<'_, '_>) -> bool {
        value.into_option().is_ok_and(facet::PeekOption::is_none)
    }

    fn serialize_opaque_scalar_with_field(
        &mut self,
        _field: Option<&facet::Field>,
        shape: &'static Shape,
        value: Peek<'_, '_>,
    ) -> Result<bool, ()> {
        if shape.id == Value::SHAPE.id {
            let json = value.get::<Value>().map_err(|_| ())?;
            self.value(json)?;
            return Ok(true);
        }
        // A value read through a proxy whose one field is a declared float (a strict float,
        // reached through a `#[facet(transparent)]` newtype or not) is written as that float, by
        // its declared width: its proxy would widen an `f32` to `f64`, and the walk unwraps a
        // newtype only after this hook. A plain float the walk writes by its type
        // (`typed_scalar`).
        let scalar = value.innermost_peek();
        if let Some((ty, float)) = proxied_float(scalar) {
            self.typed_scalar(ty, float)?;
            return Ok(true);
        }
        // A Molang constant is written as the token an author writes for it, not as its proxy's
        // widened `f64`; the other forms go through the proxy.
        if let Ok(crate::molang::Molang::Const(x)) = scalar.get::<crate::molang::Molang>() {
            match crate::molang::ConstToken::of(*x) {
                crate::molang::ConstToken::Integer(n) => self.raw(&n.to_string()),
                crate::molang::ConstToken::Real(x) => self.f32(x)?,
            }
            return Ok(true);
        }
        Ok(false)
    }

    fn raw_serialize_shape(&self) -> Option<&'static Shape> {
        Some(facet_json::RawJson::SHAPE)
    }

    fn raw_scalar(&mut self, content: &str) -> Result<(), ()> {
        self.raw(content);
        Ok(())
    }

    fn format_namespace(&self) -> Option<&'static str> {
        Some("json")
    }
}

/// The one field of `value`, a proxied struct, when it is declared a float: its scalar type and
/// its value.
fn proxied_float<'mem, 'facet>(
    value: Peek<'mem, 'facet>,
) -> Option<(ScalarType, Peek<'mem, 'facet>)> {
    let shape = value.shape();
    let Type::User(UserType::Struct(st)) = shape.ty else {
        return None;
    };
    if shape.effective_proxy(None).is_none() || st.fields.len() != 1 {
        return None;
    }
    let ty = st.fields[0].shape().scalar_type()?;
    if !matches!(ty, ScalarType::F32 | ScalarType::F64) {
        return None;
    }
    Some((ty, value.into_struct().ok()?.field(0).ok()?))
}

/// An integer scalar as facet-format normalises it.
fn integer(scalar_type: ScalarType, value: Peek<'_, '_>) -> Option<ScalarValue<'static>> {
    Some(match scalar_type {
        ScalarType::U8 => ScalarValue::U64(u64::from(*value.get::<u8>().ok()?)),
        ScalarType::U16 => ScalarValue::U64(u64::from(*value.get::<u16>().ok()?)),
        ScalarType::U32 => ScalarValue::U64(u64::from(*value.get::<u32>().ok()?)),
        ScalarType::U64 => ScalarValue::U64(*value.get::<u64>().ok()?),
        ScalarType::USize => ScalarValue::U64(u64::try_from(*value.get::<usize>().ok()?).ok()?),
        ScalarType::U128 => ScalarValue::U128(*value.get::<u128>().ok()?),
        ScalarType::I8 => ScalarValue::I64(i64::from(*value.get::<i8>().ok()?)),
        ScalarType::I16 => ScalarValue::I64(i64::from(*value.get::<i16>().ok()?)),
        ScalarType::I32 => ScalarValue::I64(i64::from(*value.get::<i32>().ok()?)),
        ScalarType::I64 => ScalarValue::I64(*value.get::<i64>().ok()?),
        ScalarType::ISize => ScalarValue::I64(i64::try_from(*value.get::<isize>().ok()?).ok()?),
        ScalarType::I128 => ScalarValue::I128(*value.get::<i128>().ok()?),
        ScalarType::Char => ScalarValue::Char(*value.get::<char>().ok()?),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use super::*;

    #[derive(Debug, Default, Facet)]
    struct Leaf {
        #[facet(rename = "minecraft:scale")]
        scale: f32,
    }

    #[derive(Debug, Default, Facet)]
    struct Tree {
        #[facet(default)]
        leaves: Vec<Leaf>,
        #[facet(default)]
        named: IndexMap<String, Leaf>,
        #[facet(default)]
        raw: Option<Value>,
    }

    /// The path and kind of the first value `tree` cannot write.
    fn failure(tree: &Tree) -> (String, Unrepresentable) {
        let (at, what) = write_in(tree, &mut Vec::new(), WriteStyle::default()).unwrap_err();
        (at.to_string(), what)
    }

    #[test]
    fn not_representable_names_the_array_element_and_member() {
        let tree = Tree {
            leaves: vec![Leaf { scale: 1.0 }, Leaf { scale: f32::NAN }],
            ..Tree::default()
        };
        assert_eq!(
            failure(&tree),
            ("leaves/1/minecraft:scale".into(), Unrepresentable::NaN)
        );
    }

    #[test]
    fn not_representable_names_the_map_key() {
        let mut tree = Tree::default();
        tree.named.insert("a".into(), Leaf { scale: 0.5 });
        tree.named.insert(
            "b".into(),
            Leaf {
                scale: f32::NEG_INFINITY,
            },
        );
        assert_eq!(
            failure(&tree),
            (
                "named/b/minecraft:scale".into(),
                Unrepresentable::NegativeInfinity
            )
        );
    }

    #[test]
    fn not_representable_names_a_member_inside_a_value() {
        let mut array = facet_value::VArray::new();
        array.push(Value::from(1i64));
        array.push(Value::from(f64::INFINITY));
        let tree = Tree {
            raw: Some(Value::from(array)),
            ..Tree::default()
        };
        assert_eq!(failure(&tree), ("raw/1".into(), Unrepresentable::Infinity));
    }

    #[test]
    fn a_failed_write_appends_nothing() {
        let mut out = b"kept".to_vec();
        let tree = Tree {
            leaves: vec![Leaf { scale: f32::NAN }],
            ..Tree::default()
        };
        assert!(write_in(&tree, &mut out, WriteStyle::default()).is_err());
        assert_eq!(out, b"kept");
    }
}
