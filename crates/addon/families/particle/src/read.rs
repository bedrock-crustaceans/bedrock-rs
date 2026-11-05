//! The reader and writer both particle structs share.
//!
//! A particle document has the same members in both eras; what differs is how a value that is
//! missing or of the wrong type is treated. The legacy parser (`format_version` below 1.20.80)
//! logs a line and goes on with the field's default; the reader of 1.20.80 and later
//! documents refuses the document. [`Era`] says which, and every type of the document reads
//! itself from a JSON value under it ([`Node`]), writes itself back, and hands its Molang
//! fields to the walk. [`ParticleEffect`] is read here as the game reads a 1.20.80 document,
//! with the parts of that read the legacy struct shares.

#![allow(
    clippy::single_match_else,
    reason = "the two arms are the two outcomes of a read: the value, or the line about it"
)]

use facet_value::{VArray, VObject};
use indexmap::IndexMap;

use super::component::{ParticleComponent, read_named};
use super::curve::{Curve, named_map, object_member};
use super::event::EventNode;
use super::{BasicRenderParameters, Description, ParticleEffect, texts};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::document::{ComponentMap, Extra};
use bedrock_addon_core::id::ParticleId;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Unrepresentable, Value};
use bedrock_addon_core::molang::{
    JsonScalar, Molang, MolangFailure, MolangFieldMut, MolangFieldSpec, MolangFields, MolangSource,
    MolangVisitor, RawVersion, ReadField, ReaderKind, Step, json_scalar,
};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned};

/// Which loader reads the document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Era {
    /// The legacy parser: below 1.20.80, a bad value is logged and replaced by its default.
    Legacy,
    /// The 1.20.80 reader: a bad value refuses the document.
    Cereal,
}

/// The Molang spec of a field whose legacy default is `default`: a legacy field that does not
/// compile becomes that constant, a 1.20.80 field and a legacy event expression are kept. In
/// every era the lines are the Molang parser's content-log errors and the effect loads, so
/// they are kept errors.
pub(crate) fn spec(era: Era, o: Opt) -> MolangFieldSpec {
    let default = o.default;
    match era {
        // The game parses an event's `expression` and ignores the result: the event is
        // kept. The crate compiles it once, on the upgraded document at the file's path,
        // under the same spec.
        Era::Legacy if o.event => KEPT.reader(ReaderKind::SchemaValidated).compile_empty(),
        // A failed parse becomes the field's default, for a single field and for an array
        // element alike, and the reader goes on.
        Era::Legacy => KEPT
            .reader(ReaderKind::SchemaValidated)
            .compile_empty()
            .on_failure(MolangFailure::ReplaceWith(default)),
        // A 1.20.80 document holds the text; the game compiles it when it builds the effect,
        // drops the parse result and registers the effect anyway: kept.
        Era::Cereal => KEPT,
    }
}

/// Every particle Molang field: its parser lines are kept errors, the effect loads.
pub(crate) const KEPT: MolangFieldSpec = MolangFieldSpec::KEPT;

/// What a field's reader needs to know about the field.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Opt {
    /// The legacy parser's default for the field: what a bad value becomes there.
    pub(crate) default: f32,
    /// The legacy parser's line for a value of the wrong type; `None` for the crate's own.
    pub(crate) wrong: Option<&'static str>,
    /// The legacy parser's line when the member is absent; `None` when it says nothing.
    pub(crate) missing: Option<&'static str>,
    /// A Molang member of an event: the legacy parser reads it only from a string, says
    /// nothing about another value, and keeps an expression that does not compile.
    pub(crate) event: bool,
}

impl Opt {
    /// A field with default 0 and no text of its own.
    pub(crate) const NONE: Self = Self::with_default(0.0);

    /// A field whose legacy default is `default`.
    pub(crate) const fn with_default(default: f32) -> Self {
        Self {
            default,
            wrong: None,
            missing: None,
            event: false,
        }
    }

    /// A Molang member of an event.
    pub(crate) const fn event(mut self) -> Self {
        self.event = true;
        self
    }

    /// A field the legacy helper is told to require: "required field does not exist".
    pub(crate) const fn required(default: f32) -> Self {
        Self::with_default(default).missing(texts::REQUIRED_MISSING)
    }

    /// The same field with `text` for a value of the wrong type.
    pub(crate) const fn wrong(mut self, text: &'static str) -> Self {
        self.wrong = Some(text);
        self
    }

    /// The same field with `text` for an absent member.
    pub(crate) const fn missing(mut self, text: &'static str) -> Self {
        self.missing = Some(text);
        self
    }
}

/// Where a read reports, and under which era.
pub(crate) struct Reader<'d> {
    /// The era of the document.
    era: Era,
    /// The pack's Molang version: what a plain string compiles at.
    molang_version: RawVersion,
    path: JsonPath,
    muted: u32,
    d: &'d mut Diagnostics,
}

impl<'d> Reader<'d> {
    /// A reader reporting below `at`.
    pub(crate) fn new(
        era: Era,
        molang_version: RawVersion,
        at: JsonPath,
        d: &'d mut Diagnostics,
    ) -> Self {
        Self {
            era,
            molang_version,
            path: at,
            muted: 0,
            d,
        }
    }

    /// Whether the legacy parser reads the document.
    pub(crate) fn legacy(&self) -> bool {
        self.era == Era::Legacy
    }

    /// A finding at the current path.
    pub(crate) fn push(&mut self, code: DiagnosticCode, severity: Severity) {
        if self.muted == 0 {
            self.d.push(
                Diagnostic::error(code)
                    .severity(severity)
                    .at(self.path.clone()),
            );
        }
    }

    /// A content-log line of vanilla's, written by the legacy parser and by the reader of
    /// 1.20.80 documents: the document loads and the value takes its fallback, so the finding
    /// is a kept error.
    pub(crate) fn warn(&mut self, text: &'static str) {
        self.content_error(text);
    }

    /// A value the crate changes without a line of vanilla's: an information.
    pub(crate) fn warn_ours(&mut self) {
        self.push(DiagnosticCode::InvalidValue { text: None }, Severity::Info);
    }

    /// A finding that refuses the document.
    pub(crate) fn error(&mut self, code: DiagnosticCode) {
        self.push(code, Severity::Error);
    }

    /// A value of the wrong JSON type: a 1.20.80 document is refused; the legacy parser logs
    /// `text` (when it has one) and the caller takes the default.
    pub(crate) fn wrong_type(&mut self, text: Option<&'static str>) {
        match (self.era, text) {
            (Era::Cereal, _) => self.error(DiagnosticCode::InvalidType { text: None }),
            (Era::Legacy, Some(text)) => self.warn(text),
            (Era::Legacy, None) => self.warn_ours(),
        }
    }

    /// A required member that is absent, reported at the current path (the member's): the
    /// 1.20.80 document is refused; the legacy parser logs `text` when it has one.
    pub(crate) fn missing(&mut self, text: Option<&'static str>) {
        match (self.era, text) {
            (Era::Cereal, _) => self.error(DiagnosticCode::MissingRequiredField { text: None }),
            (Era::Legacy, Some(text)) => self.warn(text),
            (Era::Legacy, None) => {}
        }
    }

    /// Runs `f` below the member `key`.
    pub(crate) fn key<R>(&mut self, key: &str, f: impl FnOnce(&mut Self) -> R) -> R {
        self.path.push(key);
        let result = f(self);
        self.path.pop();
        result
    }

    /// Runs `f` below the element `index`.
    pub(crate) fn index<R>(&mut self, index: usize, f: impl FnOnce(&mut Self) -> R) -> R {
        self.path.push_index(index);
        let result = f(self);
        self.path.pop();
        result
    }

    /// Records that the list at the current path holds, in order, the elements the file has
    /// at the indices `kept`: what is told of an element from here on, by the load and by a
    /// rule that reads the effect later, is at the index the file has. Called when all that
    /// the reader tells of the list at the file's indices is told.
    pub(crate) fn kept(&mut self, kept: &[usize]) {
        if self.muted == 0 {
            self.d.left_out(&self.path, kept);
        }
    }

    /// Takes back what the read of the member `key` told, for a member the legacy parser
    /// never reads (it returned before it); then tells that it is left out, a line of the
    /// crate's own: an information.
    pub(crate) fn unread(&mut self, key: &str) {
        let at = self.path.child(key);
        self.d
            .retain(|finding| !finding.location.json_path.starts_with(&at));
        self.key(key, |r| {
            r.push(DiagnosticCode::InvalidValue { text: None }, Severity::Info);
        });
    }

    /// A line vanilla's legacy parser writes to the content log at the error level before its
    /// parse returns and the effect loads: a kept error, which rejects nothing.
    pub(crate) fn content_error(&mut self, text: &'static str) {
        if self.muted == 0 {
            self.d.push_kept_error(
                Diagnostic::error(DiagnosticCode::InvalidValue {
                    text: Some(VanillaText::Fixed(text)),
                })
                .at(self.path.clone()),
            );
        }
    }

    /// `code` as a kept error at the current path.
    pub(crate) fn push_kept(&mut self, code: DiagnosticCode) {
        if self.muted == 0 {
            self.d
                .push_kept_error(Diagnostic::error(code).at(self.path.clone()));
        }
    }

    /// Runs `f` without reporting: the defaults of a component whose value is no object.
    pub(crate) fn silently<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.muted += 1;
        let result = f(self);
        self.muted -= 1;
        result
    }
}

/// The members of an object being read.
#[derive(Clone, Copy)]
pub(crate) struct Members<'v>(Option<&'v VObject>);

impl<'v> Members<'v> {
    /// No members: what a component that is no object reads its defaults from.
    pub(crate) const EMPTY: Self = Self(None);

    /// The members of `value`, when it is an object.
    pub(crate) fn of(value: &'v Value) -> Option<Self> {
        value.as_object().map(|object| Self(Some(object)))
    }

    /// The member `key`.
    pub(crate) fn get(self, key: &str) -> Option<&'v Value> {
        self.0?.get(key)
    }

    /// Every member, in file order.
    pub(crate) fn iter(self) -> impl Iterator<Item = (&'v str, &'v Value)> {
        self.0
            .into_iter()
            .flat_map(|object| object.iter().map(|(key, value)| (key.as_str(), value)))
    }

    /// The members `known` does not name: each reported as `UnrecognizedField` (a warning,
    /// which a 1.20.80 document makes an error; the legacy parser reads past them) and not
    /// kept: the game never stores an unknown key, so the value holds none and the upgrade
    /// carries none. The `extra` of a value is for one built by hand.
    pub(crate) fn extra(self, known: fn(&str) -> bool, r: &mut Reader<'_>) -> Extra {
        for (key, _) in self.iter() {
            if !known(key) {
                // The legacy parser reads past an unknown member without a word, so the
                // crate reports it as an information. A 1.20.80 document logs it as a
                // warning and the strict loading of that format makes it an error that
                // refuses the document.
                let severity = if r.legacy() {
                    Severity::Info
                } else {
                    Severity::Warning
                };
                r.key(key, |r| {
                    r.push(DiagnosticCode::UnrecognizedField { text: None }, severity);
                });
            }
        }
        Extra::default()
    }
}

/// Where a write is, for the path of a value that has no JSON form.
#[derive(Default)]
pub(crate) struct Writer {
    path: JsonPath,
    /// Whether the document written is of the legacy era, whose reader takes a boolean at a
    /// Molang member.
    legacy: bool,
}

impl Writer {
    /// A writer of a document of the legacy era (below 1.20.80).
    pub(crate) fn new_legacy() -> Self {
        Self {
            legacy: true,
            ..Self::default()
        }
    }

    /// The error of a value at the current path that cannot be written.
    pub(crate) fn refuse(&self, what: Unrepresentable) -> EncodeError {
        EncodeError::NotRepresentable {
            at: self.path.clone(),
            what,
        }
    }

    /// Runs `f` below the member `key`.
    pub(crate) fn key<R>(&mut self, key: &str, f: impl FnOnce(&mut Self) -> R) -> R {
        self.path.push(key);
        let result = f(self);
        self.path.pop();
        result
    }

    /// Runs `f` below the element `index`.
    pub(crate) fn index<R>(&mut self, index: usize, f: impl FnOnce(&mut Self) -> R) -> R {
        self.path.push_index(index);
        let result = f(self);
        self.path.pop();
        result
    }

    /// How deep the current path is.
    pub(crate) fn depth(&self) -> usize {
        self.path.len()
    }

    /// Adds the unknown members after the known ones. A member named like a known one would
    /// replace it in the text, so it has no place there; in a 1.20.80 document no unknown
    /// member has one, since the strict mask refuses it.
    pub(crate) fn extra(
        &mut self,
        out: &mut VObject,
        extra: &Extra,
        known: fn(&str) -> bool,
    ) -> Result<(), EncodeError> {
        for (key, value) in extra.iter() {
            if !self.legacy || known(key) || out.contains_key(key) {
                return Err(self.key(key, |w| w.refuse(Unrepresentable::Unsupported)));
            }
            out.insert(key, value.clone());
        }
        Ok(())
    }
}

/// A value of a particle document: read from JSON under an era, written back, walked for its
/// Molang.
pub(crate) trait Node: Sized {
    /// Reads `value` at the reader's path. `None` when the value cannot be used: a 1.20.80
    /// document has been refused, or the legacy parser has logged its line and the caller
    /// goes on with the field's default.
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self>;

    /// What a required field holds when the member is absent or unusable.
    fn fallback(o: Opt) -> Self;

    /// The JSON of the value.
    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError>;

    /// Hands the Molang fields to the walk; `o` is the field's.
    fn walk(&mut self, _era: Era, _o: Opt, _v: &mut MolangVisitor<'_>) {}
}

/// An object of a particle document: a component or a part of one.
pub(crate) trait Fields: Sized {
    /// Whether the object has a member `key`.
    fn known(key: &str) -> bool;
    /// Reads the members; an absent or unusable one takes its default.
    fn read_members(m: Members<'_>, r: &mut Reader<'_>) -> Self;
    /// Writes the members into `out`.
    fn write_members(&self, out: &mut VObject, w: &mut Writer) -> Result<(), EncodeError>;
    /// Hands the Molang fields to the walk.
    fn walk_members(&mut self, era: Era, v: &mut MolangVisitor<'_>);
}

impl<T: Fields> Node for T {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        match Members::of(value) {
            Some(members) => Some(T::read_members(members, r)),
            None => {
                r.wrong_type(o.wrong);
                None
            }
        }
    }

    fn fallback(_: Opt) -> Self {
        // The defaults, read from no members without a line.
        let mut scratch = Diagnostics::default();
        let mut r = Reader::new(Era::Legacy, RawVersion(0), JsonPath::new(), &mut scratch);
        r.silently(|r| T::read_members(Members::EMPTY, r))
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        let mut out = VObject::new();
        self.write_members(&mut out, w)?;
        Ok(out.into())
    }

    fn walk(&mut self, era: Era, _: Opt, v: &mut MolangVisitor<'_>) {
        self.walk_members(era, v);
    }
}

/// An optional member.
pub(crate) fn opt<T: Node>(m: Members<'_>, key: &str, o: Opt, r: &mut Reader<'_>) -> Option<T> {
    let value = m.get(key)?;
    r.key(key, |r| T::read(value, r, o))
}

/// A required member; the field's fallback when it is absent or unusable.
pub(crate) fn req<T: Node>(m: Members<'_>, key: &str, o: Opt, r: &mut Reader<'_>) -> T {
    r.key(key, |r| match m.get(key) {
        Some(value) => T::read(value, r, o),
        None => {
            r.missing(o.missing);
            None
        }
    })
    .unwrap_or_else(|| T::fallback(o))
}

/// Writes a required member.
pub(crate) fn put<T: Node>(
    out: &mut VObject,
    key: &str,
    value: &T,
    w: &mut Writer,
) -> Result<(), EncodeError> {
    let json = w.key(key, |w| value.write(w))?;
    out.insert(key, json);
    Ok(())
}

/// Writes an optional member when the document has it.
pub(crate) fn put_opt<T: Node>(
    out: &mut VObject,
    key: &str,
    value: Option<&T>,
    w: &mut Writer,
) -> Result<(), EncodeError> {
    match value {
        Some(value) => put(out, key, value, w),
        None => Ok(()),
    }
}

/// Walks a member.
pub(crate) fn walk<T: Node>(key: &str, value: &mut T, era: Era, o: Opt, v: &mut MolangVisitor<'_>) {
    v.enter(Step::Key(key));
    value.walk(era, o, v);
    v.leave();
}

/// The `f32` of a JSON number of either tag. A number whose nearest `f32` is infinite is read
/// as the largest finite float of its sign, with a warning.
pub(crate) fn float(value: &Value, r: &mut Reader<'_>) -> Option<f32> {
    let number = value.as_number()?.to_f64_lossy();
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the nearest f32 is the reading"
    )]
    let narrowed = number as f32;
    if narrowed.is_finite() {
        return Some(narrowed);
    }
    r.warn_ours();
    Some(if number.is_nan() {
        0.0
    } else if number < 0.0 {
        f32::MIN
    } else {
        f32::MAX
    })
}

/// The JSON number of an `f32`: the shortest decimal that reads back as it.
pub(crate) fn float_value(x: f32, w: &Writer) -> Result<Value, EncodeError> {
    if x.is_nan() {
        return Err(w.refuse(Unrepresentable::NaN));
    }
    if x.is_infinite() {
        return Err(w.refuse(if x > 0.0 {
            Unrepresentable::Infinity
        } else {
            Unrepresentable::NegativeInfinity
        }));
    }
    let widened = f64::from(x);
    Ok(Value::from(
        format!("{x:?}").parse::<f64>().unwrap_or(widened),
    ))
}

impl Node for f32 {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        let read = float(value, r);
        if read.is_none() {
            r.wrong_type(o.wrong);
        }
        read
    }

    fn fallback(o: Opt) -> Self {
        o.default
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        float_value(*self, w)
    }
}

impl Node for bool {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        let read = value.as_bool();
        if read.is_none() {
            r.wrong_type(o.wrong);
        }
        read
    }

    fn fallback(_: Opt) -> Self {
        false
    }

    fn write(&self, _: &mut Writer) -> Result<Value, EncodeError> {
        Ok(Value::from(*self))
    }
}

impl Node for i32 {
    /// An integer token, or a real one that holds an integer.
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        let read = value.as_number().and_then(|number| {
            let x = number.to_f64_lossy();
            #[allow(clippy::cast_possible_truncation, reason = "integral and in range")]
            (x.fract() == 0.0 && x >= f64::from(i32::MIN) && x <= f64::from(i32::MAX))
                .then_some(x as i32)
        });
        if read.is_none() {
            r.wrong_type(o.wrong);
        }
        read
    }

    fn fallback(o: Opt) -> Self {
        #[allow(clippy::cast_possible_truncation, reason = "a small constant")]
        let default = o.default as i32;
        default
    }

    fn write(&self, _: &mut Writer) -> Result<Value, EncodeError> {
        Ok(Value::from(i64::from(*self)))
    }
}

impl Node for String {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        let read = value.as_string().map(|text| text.as_str().to_owned());
        if read.is_none() {
            r.wrong_type(o.wrong);
        }
        read
    }

    fn fallback(_: Opt) -> Self {
        String::new()
    }

    fn write(&self, _: &mut Writer) -> Result<Value, EncodeError> {
        Ok(Value::from(self.as_str()))
    }
}

/// A Molang value read where the legacy helper would log `text` for a bad one.
fn molang(value: &Value, r: &mut Reader<'_>, o: Opt, text: &'static str) -> Option<Molang> {
    // An event's Molang is read by the legacy parser only from a string.
    if value.is_number() && !(o.event && r.legacy()) {
        return float(value, r).map(Molang::Const);
    }
    match r.era {
        // The legacy parser takes a number or a string here; anything else is logged and the
        // field's default stands.
        Era::Legacy if o.event => match json_scalar(value) {
            JsonScalar::String(text) => {
                Some(Molang::Expr(MolangSource::string(text, r.molang_version)))
            }
            _ => {
                // Read as the member's default, 0, without a line of the game's: the crate
                // notes it at the information level. A `null` member is an absent one.
                if !value.is_null() {
                    r.push(DiagnosticCode::InvalidType { text: None }, Severity::Info);
                }
                None
            }
        },
        Era::Legacy => Some(match json_scalar(value) {
            JsonScalar::Bool(b) => Molang::Bool(b),
            JsonScalar::String(text) => Molang::Expr(MolangSource::string(text, r.molang_version)),
            _ => {
                r.warn(text);
                Molang::Const(o.default)
            }
        }),
        Era::Cereal => {
            match bedrock_addon_core::molang::read(
                value,
                ReaderKind::StrictVersioned,
                r.molang_version,
            ) {
                Ok(ReadField::Value(value)) => Some(value),
                _ => {
                    r.error(DiagnosticCode::InvalidType { text: None });
                    None
                }
            }
        }
    }
}

/// Writes a Molang value in the form it was read. A bool is written as it is in a document
/// of the legacy era, whose reader takes it; it has no form a 1.20.80 document reads.
fn molang_value(value: &Molang, w: &Writer) -> Result<Value, EncodeError> {
    if let Molang::Bool(flag) = value {
        return if w.legacy {
            Ok(Value::from(*flag))
        } else {
            Err(w.refuse(Unrepresentable::Unsupported))
        };
    }
    bedrock_addon_core::molang::write(value).map_err(|what| w.refuse(what))
}

impl Node for Molang {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        molang(value, r, o, texts::NOT_MOLANG)
    }

    fn fallback(o: Opt) -> Self {
        Molang::Const(o.default)
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        molang_value(self, w)
    }

    fn walk(&mut self, era: Era, o: Opt, v: &mut MolangVisitor<'_>) {
        v.emit(&spec(era, o), MolangFieldMut::Value(self));
    }
}

impl<const N: usize> Node for [Molang; N] {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        // The legacy parser takes an array of exactly N, or logs "error reading array" and
        // leaves every element at the default.
        let elements = value.as_array().filter(|elements| elements.len() == N);
        let Some(elements) = elements else {
            if r.legacy() {
                r.warn(texts::ARRAY_ERROR);
                return Some(Self::fallback(o));
            }
            r.error(DiagnosticCode::InvalidType { text: None });
            return None;
        };
        let mut complete = true;
        let read = std::array::from_fn(|i| {
            let element = &elements[i];
            let read = r.index(i, |r| {
                if r.legacy() && element.is_null() {
                    r.warn(texts::ARRAY_ELEMENT_MISSING);
                    return Some(Molang::Const(o.default));
                }
                molang(element, r, o, texts::ARRAY_ELEMENT_NOT_MOLANG)
            });
            complete &= read.is_some();
            read.unwrap_or(Molang::Const(o.default))
        });
        complete.then_some(read)
    }

    fn fallback(o: Opt) -> Self {
        std::array::from_fn(|_| Molang::Const(o.default))
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        let mut out = VArray::new();
        for (i, element) in self.iter().enumerate() {
            out.push(w.index(i, |w| molang_value(element, w))?);
        }
        Ok(out.into())
    }

    fn walk(&mut self, era: Era, o: Opt, v: &mut MolangVisitor<'_>) {
        let spec = spec(era, o);
        for (i, element) in self.iter_mut().enumerate() {
            v.enter(Step::Index(i));
            v.emit(&spec, MolangFieldMut::Value(element));
            v.leave();
        }
    }
}

impl<const N: usize> Node for [f32; N] {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        let elements = value.as_array().filter(|elements| elements.len() == N);
        let Some(elements) = elements else {
            r.wrong_type(o.wrong);
            return None;
        };
        let mut complete = true;
        let read = std::array::from_fn(|i| {
            let read = r.index(i, |r| f32::read(&elements[i], r, o));
            complete &= read.is_some();
            read.unwrap_or(o.default)
        });
        complete.then_some(read)
    }

    fn fallback(o: Opt) -> Self {
        [o.default; N]
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        let mut out = VArray::new();
        for (i, element) in self.iter().enumerate() {
            out.push(w.index(i, |w| float_value(*element, w))?);
        }
        Ok(out.into())
    }
}

/// A list of nodes: an array, each element read on its own. In the legacy era an element
/// that cannot be used is left out, and the reader records where the others are in the file.
impl<T: Node> Node for Vec<T> {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        let (read, places) = read_list(value, r, o)?;
        r.kept(&places);
        Some(read)
    }

    fn fallback(_: Opt) -> Self {
        Vec::new()
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        let mut out = VArray::new();
        for (i, element) in self.iter().enumerate() {
            out.push(w.index(i, |w| element.write(w))?);
        }
        Ok(out.into())
    }

    fn walk(&mut self, era: Era, o: Opt, v: &mut MolangVisitor<'_>) {
        for (i, element) in self.iter_mut().enumerate() {
            v.enter(Step::Index(i));
            element.walk(era, o, v);
            v.leave();
        }
    }
}

/// The elements of an array that can be used, each read on its own, and the index each has
/// in the file. `None`, with the finding of a wrong type, for a value that is no array. The
/// caller records the indices ([`Reader::kept`]) once it has told what it tells of the list.
pub(crate) fn read_list<T: Node>(
    value: &Value,
    r: &mut Reader<'_>,
    o: Opt,
) -> Option<(Vec<T>, Vec<usize>)> {
    let Some(elements) = value.as_array() else {
        r.wrong_type(o.wrong);
        return None;
    };
    let mut read = Vec::with_capacity(elements.len());
    let mut places = Vec::with_capacity(elements.len());
    for (i, element) in elements.iter().enumerate() {
        if let Some(element) = r.index(i, |r| T::read(element, r, o)) {
            read.push(element);
            places.push(i);
        }
    }
    Some((read, places))
}

/// A map of nodes keyed by a string, in file order; an entry that cannot be used is left out
/// in the legacy era.
impl<T: Node> Node for indexmap::IndexMap<String, T> {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        let Some(members) = Members::of(value) else {
            r.wrong_type(o.wrong);
            return None;
        };
        let mut read = Self::new();
        for (key, entry) in members.iter() {
            if let Some(entry) = r.key(key, |r| T::read(entry, r, o)) {
                read.insert(key.to_owned(), entry);
            }
        }
        Some(read)
    }

    fn fallback(_: Opt) -> Self {
        Self::new()
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        let mut out = VObject::new();
        for (key, entry) in self {
            put(&mut out, key, entry, w)?;
        }
        Ok(out.into())
    }

    fn walk(&mut self, era: Era, o: Opt, v: &mut MolangVisitor<'_>) {
        for (key, entry) in self.iter_mut() {
            walk(key, entry, era, o, v);
        }
    }
}

/// Declares an object of a particle document: the struct, its reader, its writer, its Molang
/// walk and an accessor for each member that has a default.
///
/// A member is `req` (the struct holds the value) or `opt` (it holds an `Option`, `None` when
/// the document does not write the member); after its JSON key comes its [`Opt`] and, for an
/// `opt` member, `default <expr>` to generate the accessor. `finish = <fn>` names a hook that
/// runs after the members are read.
macro_rules! object {
    (
        $(#[$meta:meta])*
        pub struct $name:ident $(finish = $finish:path)? {
            $(
                $(#[$fmeta:meta])*
                $kind:ident $field:ident : $ty:ty = $json:literal, $opt:expr $(, default $default:expr)? ;
            )*
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name {
            $(
                $(#[$fmeta])*
                pub $field: object!(@ty $kind $ty),
            )*
            /// Unknown members.
            pub extra: ::bedrock_addon_core::document::Extra,
        }

        impl $name {
            $($(
                #[doc = concat!("JSON `", $json, "`, or its default when the document does not write it.")]
                pub fn $field(&self) -> $ty {
                    self.$field.clone().unwrap_or_else(|| $default)
                }
            )?)*
        }

        /// No optional member; each required one at the value the legacy parser gives it
        /// when the document does not write it.
        impl Default for $name {
            fn default() -> Self {
                <Self as $crate::read::Node>::fallback($crate::read::Opt::NONE)
            }
        }

        impl ::bedrock_addon_core::molang::MolangFields for $name {
            fn visit_molang(&mut self, v: &mut ::bedrock_addon_core::molang::MolangVisitor<'_>) {
                $crate::read::Fields::walk_members(
                    self,
                    $crate::read::Era::Cereal,
                    v,
                );
            }
        }

        impl $crate::read::Fields for $name {
            fn known(key: &str) -> bool {
                $(if key == $json {
                    return true;
                })*
                let _ = key;
                false
            }

            #[allow(unused_variables, unused_mut)]
            fn read_members(
                m: $crate::read::Members<'_>,
                r: &mut $crate::read::Reader<'_>,
            ) -> Self {
                let mut read = Self {
                    $($field: object!(@read $kind m, $json, $opt, r),)*
                    extra: m.extra(<Self as $crate::read::Fields>::known, r),
                };
                $($finish(&mut read, m, r);)?
                read
            }

            #[allow(unused_variables)]
            fn write_members(
                &self,
                out: &mut facet_value::VObject,
                w: &mut $crate::read::Writer,
            ) -> Result<(), ::bedrock_addon_core::json::EncodeError> {
                $(object!(@write $kind out, $json, self.$field, w);)*
                w.extra(out, &self.extra, <Self as $crate::read::Fields>::known)
            }

            #[allow(unused_variables)]
            fn walk_members(
                &mut self,
                era: $crate::read::Era,
                v: &mut ::bedrock_addon_core::molang::MolangVisitor<'_>,
            ) {
                $(object!(@walk $kind $json, self.$field, era, $opt, v);)*
            }
        }
    };
    (@ty req $ty:ty) => { $ty };
    (@ty opt $ty:ty) => { Option<$ty> };
    (@read req $m:ident, $json:literal, $opt:expr, $r:ident) => {
        $crate::read::req($m, $json, $opt, $r)
    };
    (@read opt $m:ident, $json:literal, $opt:expr, $r:ident) => {
        $crate::read::opt($m, $json, $opt, $r)
    };
    (@write req $out:ident, $json:literal, $value:expr, $w:ident) => {
        $crate::read::put($out, $json, &$value, $w)?
    };
    (@write opt $out:ident, $json:literal, $value:expr, $w:ident) => {
        $crate::read::put_opt($out, $json, $value.as_ref(), $w)?
    };
    (@walk req $json:literal, $value:expr, $era:ident, $opt:expr, $v:ident) => {
        $crate::read::walk($json, &mut $value, $era, $opt, $v)
    };
    (@walk opt $json:literal, $value:expr, $era:ident, $opt:expr, $v:ident) => {
        if let Some(value) = &mut $value {
            $crate::read::walk($json, value, $era, $opt, $v);
        }
    };
}
pub(crate) use object;

// ---- the 1.20.80 document, and the parts of its read the legacy struct shares ----------

/// The first format version of the newest struct: from this version the envelope is checked
/// and the document is read strictly.
pub(crate) const CEREAL: FormatVersion = FormatVersion::new(1, 20, 80);

/// The component the 1.26.30 engine added to the legacy registry only: its parser reads
/// nothing, and the upgrade drops it.
pub(crate) const CAMERA_DISTANCE_FADE: &str = "minecraft:particle_appearance_camera_distance_fade";

/// The longest `texture` a 1.20.80 document takes.
pub(crate) const MAX_TEXTURE: usize = 256;

impl Node for ParticleId {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        String::read(value, r, o).map(Self::parse)
    }

    fn fallback(_: Opt) -> Self {
        Self::parse(String::new())
    }

    fn write(&self, _: &mut Writer) -> Result<Value, EncodeError> {
        Ok(Value::from(self.as_str()))
    }
}

/// The length constraint of `texture`. Only a 1.20.80 document has it, and only its read
/// comes here: the legacy description is read member by member.
pub(super) fn render_parameters(
    parameters: &mut BasicRenderParameters,
    _: Members<'_>,
    r: &mut Reader<'_>,
) {
    if parameters.texture.chars().count() > MAX_TEXTURE {
        r.key("texture", |r| {
            r.error(DiagnosticCode::InvalidValue { text: None });
        });
    }
}

/// One entry of `components` as read.
#[allow(
    clippy::large_enum_variant,
    reason = "handed straight on to the map that holds it; nearly every entry is typed"
)]
pub(crate) enum Entry {
    /// One of the 29 components.
    Typed(ParticleComponent),
    /// `minecraft:particle_appearance_camera_distance_fade` in a legacy document, as written.
    CameraDistanceFade(Value),
}

/// Reads the entries of `components`, each handed to `sink` under its key, in file order.
pub(crate) fn read_components(
    components: Members<'_>,
    r: &mut Reader<'_>,
    mut sink: impl FnMut(&str, Entry),
) {
    for (key, value) in components.iter() {
        r.key(key, |r| {
            if key.is_empty() && r.legacy() {
                r.warn(texts::COMPONENT_WITHOUT_NAME);
                return;
            }
            match read_named(key, value, r) {
                Some(Some(component)) => sink(key, Entry::Typed(component)),
                // The 1.20.80 reader refused it; the finding rejects the document.
                Some(None) => {}
                None if r.legacy() && key == CAMERA_DISTANCE_FADE => {
                    sink(key, Entry::CameraDistanceFade(value.clone()));
                }
                None => {
                    // The game logs this as a warning in both eras; the legacy parser's
                    // line names the key.
                    let text = r.legacy().then(|| VanillaText::Formatted {
                        template: texts::UNRECOGNIZED_COMPONENT,
                        args: vec![key.to_owned()],
                    });
                    // Not kept: the game stores no unknown key. The legacy parser writes its
                    // line at the error level and the effect loads, so the finding is a kept
                    // error; a 1.20.80 document logs a warning, which its strict loading
                    // makes an error that refuses the document.
                    if r.legacy() {
                        r.push_kept(DiagnosticCode::UnrecognizedComponent { text });
                    } else {
                        r.push(
                            DiagnosticCode::UnrecognizedComponent { text },
                            Severity::Warning,
                        );
                    }
                }
            }
        });
    }
}

/// Reads `curves`.
pub(crate) fn read_curves(m: Members<'_>, r: &mut Reader<'_>) -> Option<IndexMap<String, Curve>> {
    named_map(m, "curves", texts::NO_CURVES, texts::CURVE_WITHOUT_NAME, r)
}

/// Reads `events`.
pub(crate) fn read_events(
    m: Members<'_>,
    r: &mut Reader<'_>,
) -> Option<IndexMap<String, EventNode>> {
    named_map(m, "events", texts::NO_EVENTS, texts::EVENT_WITHOUT_NAME, r)
}

/// The members of `particle_effect`.
pub(crate) fn known(key: &str) -> bool {
    matches!(key, "description" | "components" | "curves" | "events")
}

/// `text` as the JSON value the readers take. Every number token is a number in it: the
/// crate's JSON reader reads an integer literal too long for 64 bits as the double it spells
/// (nothing else needs to inspect the text first).
pub(crate) fn value_of(text: &str) -> Result<Value, facet_json::DeserializeError> {
    bedrock_addon_core::json::from_str_jsonc::<Value>(text)
}

/// The text of a version struct as JSON; a finding and `None` when it is none.
pub(crate) fn json(text: JsonSlice<'_>, at: &JsonPath, d: &mut Diagnostics) -> Option<Value> {
    match value_of(text.text) {
        Ok(value) => Some(value),
        Err(error) => {
            d.push(
                Diagnostic::error(DiagnosticCode::MalformedInput {
                    message: error.kind.to_string(),
                    text: None,
                })
                .at(at.clone()),
            );
            None
        }
    }
}

impl ParticleEffect {
    /// Reads the payload object as the game reads a 1.20.80 document. A member that cannot
    /// be used is reported as an error, which refuses the document; the value returned then
    /// only carries what could be read.
    fn read(payload: &Value, r: &mut Reader<'_>) -> Option<Self> {
        let Some(m) = Members::of(payload) else {
            r.error(DiagnosticCode::InvalidType { text: None });
            return None;
        };
        let description: Description = req(m, "description", Opt::NONE, r);
        let mut components = ComponentMap::new();
        if let Some(entries) = object_member(m, "components", r) {
            r.key("components", |r| {
                read_components(entries, r, |key, entry| match entry {
                    Entry::Typed(component) => {
                        components.insert_typed(component, false);
                    }
                    Entry::CameraDistanceFade(json) => {
                        components.insert_unknown(key, json);
                    }
                });
            });
        }
        Some(Self {
            description,
            components,
            curves: read_curves(m, r),
            events: read_events(m, r),
            extra: m.extra(known, r),
        })
    }
}

impl Versioned for ParticleEffect {
    const VERSION: FormatVersion = CEREAL;
}

impl VersionStruct for ParticleEffect {
    /// Reads the payload (from 1.20.80 the envelope is checked, so the slice is the value of
    /// `particle_effect`).
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let value = json(payload, &cx.at, d)?;
        let mut r = Reader::new(Era::Cereal, cx.load.molang_version, cx.at.clone(), d);
        Self::read(&value, &mut r)
    }

    /// The payload (`write::document_payload`).
    fn write(&self) -> Result<Value, EncodeError> {
        super::write::document_payload(self)
    }
}

impl MolangFields for ParticleEffect {
    fn visit_molang(&mut self, v: &mut MolangVisitor<'_>) {
        v.nested("components", &mut self.components);
        walk_named(&mut self.curves, &mut self.events, Era::Cereal, v);
    }
}

/// Walks `curves` and `events`.
pub(crate) fn walk_named(
    curves: &mut Option<IndexMap<String, Curve>>,
    events: &mut Option<IndexMap<String, EventNode>>,
    era: Era,
    v: &mut MolangVisitor<'_>,
) {
    if let Some(curves) = curves {
        walk("curves", curves, era, Opt::NONE, v);
    }
    if let Some(events) = events {
        walk("events", events, era, Opt::NONE, v);
    }
}

/// Walks a typed entry of a legacy `components` map under its key.
pub(crate) fn walk_legacy_component(
    key: &str,
    component: &mut ParticleComponent,
    v: &mut MolangVisitor<'_>,
) {
    v.enter(Step::Key(key));
    super::component::walk_component(component, Era::Legacy, v);
    v.leave();
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use super::*;
    use crate::component::{DistanceEvent, VectorEvents};
    use crate::linear::{array_of, assert_reads_linear, object_of, reading};

    /// The map reader alone, as `emitter_lifetime_events/timeline` and
    /// `particle_lifetime_events/timeline` use it, on a value parsed beforehand.
    #[test]
    fn the_map_reader_reads_a_timeline_of_100_000_entries_in_linear_time() {
        for era in [Era::Legacy, Era::Cereal] {
            assert_reads_linear(
                &format!("timeline ({era:?})"),
                |count| object_of(count, "0.", r#"["a", "b"]"#),
                |timeline, count| {
                    let (read, found) = reading(era, |r| {
                        IndexMap::<String, VectorEvents>::read(timeline, r, Opt::NONE)
                    });
                    assert_eq!(read.as_ref().map(IndexMap::len), Some(count));
                    assert!(found.is_empty());
                    (read, found)
                },
            );
        }
    }

    /// The list reader alone, as `looping_travel_distance_events` uses it.
    #[test]
    fn the_list_reader_reads_100_000_distance_events_in_linear_time() {
        for era in [Era::Legacy, Era::Cereal] {
            assert_reads_linear(
                &format!("distance events ({era:?})"),
                |count| array_of(count, r#"{"distance": 1, "effects": "a"}"#),
                |events, count| {
                    let (read, found) =
                        reading(era, |r| Vec::<DistanceEvent>::read(events, r, Opt::NONE));
                    assert_eq!(read.as_ref().map(Vec::len), Some(count));
                    assert!(found.is_empty());
                    (read, found)
                },
            );
        }
    }

    /// `read_components` alone on a `components` object parsed beforehand: every key is its
    /// own (unknown) component, which is one finding each and no entry.
    #[test]
    fn read_components_reads_100_000_component_keys_in_linear_time() {
        for era in [Era::Legacy, Era::Cereal] {
            assert_reads_linear(
                &format!("read_components ({era:?})"),
                |count| object_of(count, "demo:unknown", r#"{"a": 1}"#),
                |components, count| {
                    let members = Members::of(components).expect("an object");
                    let mut entries = Vec::with_capacity(count);
                    let ((), found) = reading(era, |r| {
                        read_components(members, r, |key, entry| {
                            entries.push((key.to_owned(), entry));
                        });
                    });
                    assert_eq!(entries.len(), 0);
                    assert_eq!(found.len(), count);
                    (entries, found)
                },
            );
        }
    }
}
