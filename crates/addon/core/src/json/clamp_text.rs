//! Numbers beyond the member that holds them, clamped in the text a typed reader is given.
//!
//! A typed member refuses a number its type cannot hold as a finite value: `1e39` in an `f32`
//! or a Molang constant, `1e999` in a value kept as JSON. Vanilla's schemas take any JSON
//! number there and read it with `asFloat`, which gives an infinity no writer can spell. Where
//! a family reads such a member, the number is read as the largest finite number of its sign
//! and width, with one `InvalidValue` warning at the number. This is the one implementation of that
//! clamp. The family says which places hold a number of which width; the
//! walk, the rewriting of the text and the finding are here.

use std::collections::HashMap;
use std::ops::Range;

use facet_json::RawJson;
use indexmap::IndexMap;

use super::Value;
use super::read::{number_end, skip_trivia, string_end};
use super::slice::{JsonSlice, parse_at};
use crate::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath, JsonPathSegment, Span};
use crate::molang::beyond_f32_warning;

/// What holds a number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Width {
    /// An `f32`: a float member, or a Molang constant.
    Float,
    /// A double: a number of a value kept as JSON.
    Double,
}

/// A text with the numbers beyond their width replaced, and where each replacement lies, so
/// that a place in it can be named as a place in the text as written.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct ClampedText {
    /// The text as written, each number beyond its width replaced by the largest finite
    /// number of its sign and width; nothing else differs.
    pub text: String,
    /// The replacements, in the order of the text.
    edits: Vec<Edit>,
}

/// One replaced number token.
#[derive(Clone, Copy, Debug)]
struct Edit {
    /// Where the token starts in the text as written.
    start: usize,
    /// The token's length as written.
    old: usize,
    /// The replacement's length.
    new: usize,
}

impl ClampedText {
    /// `span`, which a parse of [`Self::text`] at file offset `offset` gave, as the span of
    /// the same bytes in the file as written. A span that begins or ends inside a replaced
    /// number begins or ends with the number as written.
    #[must_use]
    pub fn written_span(&self, span: Span, offset: usize) -> Span {
        let start = (span.offset as usize).saturating_sub(offset);
        let end = start + span.len as usize;
        let (start, end) = (self.position(start, false), self.position(end, true));
        Span {
            offset: u32::try_from(offset + start).unwrap_or(u32::MAX),
            len: u32::try_from(end.saturating_sub(start)).unwrap_or(u32::MAX),
        }
    }

    /// Gives every span of `d`, which a reading of [`Self::text`] at file offset `offset`
    /// made, the place of the same bytes in the file as written ([`Self::written_span`]). For
    /// the findings of a whole load of the clamped text: call it once, before the findings of
    /// the clamp itself (which are of the file as written already) are added.
    pub fn restore_spans(&self, d: &mut Diagnostics, offset: usize) {
        // In place: the moves a reader recorded among the findings stay.
        for finding in d.iter_mut() {
            if let Some(span) = &mut finding.location.span {
                *span = self.written_span(*span, offset);
            }
        }
    }

    /// Ends a load that read [`Self::text`] at file offset `offset`: the findings `d` it made
    /// are given their places in the file as written ([`Self::restore_spans`]), then come
    /// the warnings `beyond` of the clamp itself ([`clamp_beyond_double`]).
    ///
    /// A number is told once: a warning of the clamp is left out where the reader
    /// has given a finding that speaks of the same number ([`spoken_for`]). The member there
    /// says what it made of the number (it holds it at a width of its own, it does not take
    /// a number, it refuses the file), and "read as the largest finite double" is then not
    /// what happened.
    ///
    /// Gives the warnings it added, for a load whose later phases may still speak of a
    /// number ([`drop_spoken_for`]).
    pub fn settle(&self, d: &mut Diagnostics, offset: usize, beyond: Diagnostics) -> Diagnostics {
        self.restore_spans(d, offset);
        let said = spoken_for(d, &beyond);
        let mut added = Diagnostics::default();
        for (finding, said) in beyond.into_iter().zip(said) {
            if !said {
                d.push(finding.clone());
                added.push(finding);
            }
        }
        added
    }

    /// The position in the text as written of `position` in the clamped text.
    fn position(&self, position: usize, is_end: bool) -> usize {
        // How much longer the clamped text is, up to the edit at hand.
        let mut grown = 0;
        let mut shrunk = 0;
        for edit in &self.edits {
            let from = edit.start + grown - shrunk;
            if position <= from {
                break;
            }
            if position < from + edit.new {
                return if is_end {
                    edit.start + edit.old
                } else {
                    edit.start
                };
            }
            grown += edit.new;
            shrunk += edit.old;
        }
        position + shrunk - grown
    }
}

/// Whether two findings are in the same file: each names the same pack and path, or one of
/// them names none.
fn same_file(theirs: &Diagnostic, ours: &Diagnostic) -> bool {
    fn same<T: PartialEq>(theirs: Option<&T>, ours: Option<&T>) -> bool {
        match (theirs, ours) {
            (Some(theirs), Some(ours)) => theirs == ours,
            _ => true,
        }
    }
    let (theirs, ours) = (&theirs.location, &ours.location);
    same(theirs.path.as_ref(), ours.path.as_ref()) && same(theirs.pack.as_ref(), ours.pack.as_ref())
}

/// Whether `finding`, at the JSON path of the clamp's `warning`, speaks of the number there:
/// it is in the same file and is about the value. A finding that only says the member's NAME
/// is not known (`UnrecognizedField`, `UnrecognizedComponent`, `UnknownSound` for an event's
/// key) is not: such a member is kept as written by the families that keep what they do not
/// read, and the number in it is then the clamp's to tell.
fn speaks_of(finding: &Diagnostic, warning: &Diagnostic) -> bool {
    same_file(finding, warning)
        && !matches!(
            finding.code,
            DiagnosticCode::UnrecognizedField { .. }
                | DiagnosticCode::UnrecognizedComponent { .. }
                | DiagnosticCode::UnknownSound { .. }
        )
}

/// Whether `finding`, at a member that encloses the number of the clamp's `warning`, says
/// that the member is not kept: a rejecting error there (the member is refused). Nothing
/// below such a member is held, so no number in it is "read as the largest finite double". A
/// warning is no such word: `DroppedComponent` at a component is also what a legacy block
/// says of a component written twice, whose first copy it keeps; nor is a kept error,
/// after which the member is held as written.
fn refuses_above(finding: &Diagnostic, warning: &Diagnostic) -> bool {
    same_file(finding, warning) && finding.rejects()
}

/// The findings of a load by their JSON path.
struct Told<'d>(HashMap<&'d JsonPath, Vec<&'d Diagnostic>>);

impl<'d> Told<'d> {
    fn of(d: &'d Diagnostics) -> Self {
        let mut told: HashMap<&JsonPath, Vec<&Diagnostic>> = HashMap::new();
        for finding in d {
            told.entry(&finding.location.json_path)
                .or_default()
                .push(finding);
        }
        Self(told)
    }

    /// Whether a finding speaks of the number `warning` is at: at its path
    /// ([`speaks_of`]), or at a member around it, the root apart ([`refuses_above`]). With
    /// `among`, `warning` is one of the findings and is not counted itself (a second
    /// finding equal to it is). The work is the depth of the path.
    fn speaks(&self, warning: &Diagnostic, among: bool) -> bool {
        let mut at = warning.location.json_path.clone();
        let mut itself = among;
        let here = self.0.get(&at).is_some_and(|there| {
            there.iter().any(|finding| {
                if itself && *finding == warning {
                    itself = false;
                    return false;
                }
                speaks_of(finding, warning)
            })
        });
        if here {
            return true;
        }
        while at.pop() && !at.is_empty() {
            let refused = self
                .0
                .get(&at)
                .is_some_and(|there| there.iter().any(|finding| refuses_above(finding, warning)));
            if refused {
                return true;
            }
        }
        false
    }
}

/// For each warning of `beyond` (the clamp's, [`clamp_beyond_double`]; none of them in `d`),
/// whether a finding of `d` speaks of the same number: one at the number's
/// JSON path that is about the value (not about a member name the reader does not know), or
/// one at a member around the number that says the member is not kept (an error); in the
/// warning's file, where both name one. One walk of `d`, and the depth of
/// its path for each warning.
#[must_use]
pub fn spoken_for(d: &Diagnostics, beyond: &Diagnostics) -> Vec<bool> {
    if beyond.is_empty() {
        return Vec::new();
    }
    let told = Told::of(d);
    beyond
        .iter()
        .map(|warning| told.speaks(warning, false))
        .collect()
}

/// `d` without the warnings of `clamp` (the clamp's, as they are in `d`) of whose number
/// another finding of `d` speaks ([`spoken_for`]): what [`ClampedText::settle`] does, for the
/// findings a load makes after it (an upgrade step, a check of the newest struct, the merge
/// of a stack's copies). One walk of `d`.
pub fn drop_spoken_for(d: &mut Diagnostics, clamp: &Diagnostics) {
    if clamp.is_empty() {
        return;
    }
    let told = Told::of(d);
    // The warnings that go, by path: each is in `d` once.
    let mut going: HashMap<JsonPath, Vec<&Diagnostic>> = HashMap::new();
    for warning in clamp {
        if told.speaks(warning, true) {
            going
                .entry(warning.location.json_path.clone())
                .or_default()
                .push(warning);
        }
    }
    if going.is_empty() {
        return;
    }
    // In place: the places of the members the load moved stay with the list.
    d.retain(|finding| {
        going
            .get_mut(&finding.location.json_path)
            .and_then(|warnings| {
                let index = warnings.iter().position(|warning| **warning == *finding)?;
                Some(warnings.swap_remove(index))
            })
            .is_none()
    });
}

/// A number beyond what holds it.
struct Wide<'v> {
    /// Where it is, from the root of the value.
    path: Vec<JsonPathSegment<'v>>,
    width: Width,
    negative: bool,
}

impl<'v> Wide<'v> {
    /// The segment `level` of its path.
    fn segment(&self, level: usize) -> Option<JsonPathSegment<'v>> {
        self.path.get(level).copied()
    }

    /// The largest finite number of the sign and width, as the shortest text that reads
    /// back to it.
    fn replacement(&self) -> &'static str {
        replacement(self.width, self.negative)
    }
}

/// The text of `slice` (whose value is `json`) with every number beyond what holds it
/// replaced by the largest finite number of its sign and width, each with an `InvalidValue`
/// warning at its path below `at`, on the number as written; `None` when no number is.
///
/// `width` says what holds the number at a path from the root of `json`: `None` where the
/// schema declares no number (the number stays as written, for the typed reader to judge).
/// It is asked only for a number whose nearest `f32` is an infinity; under [`Width::Double`]
/// a number a double holds is left. Only the number's own characters are replaced, so every
/// other character keeps its place but for the difference in length, which
/// [`ClampedText::written_span`] takes back. A text without such a number is not looked at
/// again after the walk of `json`; otherwise every object and array on the way to a number
/// is split once, so the work is linear in the text for each level of nesting.
pub fn clamp_text(
    slice: JsonSlice<'_>,
    json: &Value,
    width: &dyn Fn(&[JsonPathSegment<'_>]) -> Option<Width>,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<ClampedText> {
    let mut wide = Vec::new();
    wide_numbers(json, &mut Vec::new(), width, &mut wide);
    if wide.is_empty() {
        return None;
    }
    let mut tokens = vec![None; wide.len()];
    locate(slice, &wide, 0, 0, &mut tokens);
    let mut found: Vec<(Edit, &'static str)> = Vec::new();
    for (number, token) in wide.iter().zip(tokens) {
        let mut number_at = at.clone();
        for segment in &number.path {
            match segment {
                JsonPathSegment::Key(key) => number_at.push(key),
                JsonPathSegment::Index(index) => number_at.push_index(*index),
            }
        }
        let Some(token) = token else {
            // The `Value` holds a number here, so the text holds its token: this is not
            // reached. Were it, the finding is still made, without a span, and the
            // number stays as written for the typed parse to refuse.
            d.push(beyond_f32_warning(number_at, None));
            continue;
        };
        d.push(beyond_f32_warning(
            number_at,
            Some(Span {
                offset: u32::try_from(token.offset).unwrap_or(u32::MAX),
                len: u32::try_from(token.text.len()).unwrap_or(u32::MAX),
            }),
        ));
        let replacement = number.replacement();
        found.push((
            Edit {
                start: token.offset - slice.offset,
                old: token.text.len(),
                new: replacement.len(),
            },
            replacement,
        ));
    }
    if found.is_empty() {
        return None;
    }
    found.sort_by_key(|(edit, _)| edit.start);
    let mut text = String::with_capacity(slice.text.len() + found.len() * 24);
    let mut copied = 0;
    for (edit, replacement) in &found {
        text.push_str(slice.text.get(copied..edit.start)?);
        text.push_str(replacement);
        copied = edit.start + edit.old;
    }
    text.push_str(slice.text.get(copied..)?);
    Some(ClampedText {
        text,
        edits: found.into_iter().map(|(edit, _)| edit).collect(),
    })
}

/// [`clamp_text`] for a document whose every number is kept at the width of a double, by a
/// reader that reads its members typed from the text and keeps what it does not read (unknown
/// members, members held as JSON) for write-back: every number of `slice` that no double
/// holds (`1e999`, an integer of 400 digits) is replaced by the largest finite double of its
/// sign, with an `InvalidValue` warning at its path below `at`, on the number as written.
/// `None` when the text has no such number, which a scan of its tokens says: no
/// value is built for a text without one.
///
/// Of a member written twice every copy's number is replaced, and the warning is given for
/// the last copy, the one the readers keep ([`clamp_tokens`]).
pub fn clamp_beyond_double(
    slice: JsonSlice<'_>,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<ClampedText> {
    clamp_beyond_double_first_kept(slice, &[], at, d)
}

/// [`clamp_beyond_double`] for a reader that keeps the FIRST copy of a member written twice
/// inside the object value of a member named in `first_kept_in` ([`clamp_tokens`]): the
/// versioned load's, with `Envelope::duplicates_kept_in`.
pub fn clamp_beyond_double_first_kept(
    slice: JsonSlice<'_>,
    first_kept_in: &[&str],
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<ClampedText> {
    if !holds_beyond_double(slice.text) {
        return None;
    }
    clamp_tokens(slice, &|_| Some(Width::Double), first_kept_in, at, d)
}

/// What [`clamp_text`] does, by one walk of the tokens of `slice` instead of a value and a
/// search for each number: so it sees every copy of a member written twice, where a value
/// holds one. Every number beyond the width `width` gives for its path is replaced.
/// The warning is given for the copy a reader keeps: the last of a member written twice,
/// but the FIRST inside the object value of a member named in `first_kept_in`,
/// whose duplicates the family reads itself (`Envelope::duplicates_kept_in`: a legacy block's
/// `components`, of which it keeps the first). A number in a copy that is not kept is
/// replaced without a word: no reader holds it.
///
/// `None` when no number is replaced, and for a text that is not well-formed (the parser
/// reports that). The work is linear in the text.
pub fn clamp_tokens(
    slice: JsonSlice<'_>,
    width: &dyn Fn(&[JsonPathSegment<'_>]) -> Option<Width>,
    first_kept_in: &[&str],
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<ClampedText> {
    let hits = Walk {
        text: slice.text,
        width,
        first_kept_in,
        path: Vec::new(),
        frames: Vec::new(),
        hits: Vec::new(),
    }
    .run()?;
    if hits.is_empty() {
        return None;
    }
    let mut text = String::with_capacity(slice.text.len() + hits.len() * 24);
    let mut copied = 0;
    let mut edits = Vec::with_capacity(hits.len());
    for hit in hits {
        text.push_str(slice.text.get(copied..hit.edit.start)?);
        text.push_str(hit.replacement);
        copied = hit.edit.start + hit.edit.old;
        if let Some(path) = hit.path {
            let mut number_at = at.clone();
            for segment in &path {
                match segment {
                    Member::Key(key) => number_at.push(key),
                    Member::Index(index) => number_at.push_index(*index),
                }
            }
            d.push(beyond_f32_warning(
                number_at,
                Some(Span {
                    offset: u32::try_from(slice.offset + hit.edit.start).unwrap_or(u32::MAX),
                    len: u32::try_from(hit.edit.old).unwrap_or(u32::MAX),
                }),
            ));
        }
        edits.push(hit.edit);
    }
    text.push_str(slice.text.get(copied..)?);
    Some(ClampedText { text, edits })
}

/// A member on the way to a token.
enum Member {
    Key(String),
    Index(usize),
}

/// One number token to replace.
struct Hit {
    edit: Edit,
    replacement: &'static str,
    /// Where the number is, from the root; `None` once the copy it is in is known not to be
    /// the one kept.
    path: Option<Vec<Member>>,
}

/// An object or array the walk is in.
enum Frame {
    Object {
        /// Each key so far, with the hits of the copy of it that is kept.
        members: HashMap<String, Range<usize>>,
        /// Where the hits of the member being read start; `None` between members.
        member_from: Option<usize>,
        /// The first copy of a member is the one kept, not the last.
        first_kept: bool,
    },
    Array,
}

/// The walk of [`clamp_tokens`].
struct Walk<'a> {
    text: &'a str,
    width: &'a dyn Fn(&[JsonPathSegment<'_>]) -> Option<Width>,
    first_kept_in: &'a [&'a str],
    path: Vec<Member>,
    frames: Vec<Frame>,
    hits: Vec<Hit>,
}

impl Walk<'_> {
    /// The numbers to replace, in the order of the text: the root value's, nothing after it.
    fn run(mut self) -> Option<Vec<Hit>> {
        let bytes = self.text.as_bytes();
        let mut at = 0;
        loop {
            at = skip_trivia(bytes, at, true);
            let ended = match *bytes.get(at)? {
                b'{' => {
                    let first_kept = matches!(
                        self.path.last(),
                        Some(Member::Key(key)) if self.first_kept_in.contains(&key.as_str())
                    );
                    self.frames.push(Frame::Object {
                        members: HashMap::new(),
                        member_from: None,
                        first_kept,
                    });
                    at += 1;
                    false
                }
                b'[' => {
                    self.frames.push(Frame::Array);
                    self.path.push(Member::Index(0));
                    at += 1;
                    false
                }
                b'}' => {
                    let Some(Frame::Object {
                        member_from: None, ..
                    }) = self.frames.pop()
                    else {
                        return None;
                    };
                    at += 1;
                    true
                }
                b']' => {
                    let Some(Frame::Array) = self.frames.pop() else {
                        return None;
                    };
                    self.path.pop();
                    at += 1;
                    true
                }
                b',' => {
                    if let (Some(Frame::Array), Some(Member::Index(index))) =
                        (self.frames.last(), self.path.last_mut())
                    {
                        *index += 1;
                    }
                    at += 1;
                    false
                }
                b':' => {
                    at += 1;
                    false
                }
                b'"' => {
                    let (end, true) = string_end(bytes, at).ok()? else {
                        return None;
                    };
                    let is_key = self.key(self.text.get(at..end)?)?;
                    at = end;
                    !is_key
                }
                b'-' | b'0'..=b'9' => {
                    let end = number_end(bytes, at).max(at + 1);
                    self.number(at, self.text.get(at..end)?);
                    at = end;
                    true
                }
                _ => {
                    // `true`, `false`, `null`: whatever stands for a value.
                    let start = at;
                    while bytes.get(at).is_some_and(u8::is_ascii_alphanumeric) {
                        at += 1;
                    }
                    if at == start {
                        return None;
                    }
                    true
                }
            };
            if ended {
                self.value_ended();
                if self.frames.is_empty() {
                    return Some(self.hits);
                }
            }
        }
    }

    /// A string token: whether it is the key of a member that begins (then the path names
    /// it), not a value. `None` for a key that is no JSON string.
    fn key(&mut self, token: &str) -> Option<bool> {
        let from = self.hits.len();
        let Some(Frame::Object { member_from, .. }) = self.frames.last_mut() else {
            return Some(false);
        };
        if member_from.is_some() {
            return Some(false);
        }
        *member_from = Some(from);
        let key = if token.contains('\\') {
            crate::json::from_str::<String>(token).ok()?
        } else {
            token.get(1..token.len().checked_sub(1)?)?.to_owned()
        };
        self.path.push(Member::Key(key));
        Some(true)
    }

    /// A number token at `start`: a hit when it is beyond what holds it.
    fn number(&mut self, start: usize, token: &str) {
        let Ok(number) = token.parse::<f64>() else {
            return;
        };
        #[allow(
            clippy::cast_possible_truncation,
            reason = "the nearest f32 is the question"
        )]
        let nearest = number as f32;
        if nearest.is_finite() {
            return;
        }
        let segments: Vec<JsonPathSegment<'_>> = self
            .path
            .iter()
            .map(|member| match member {
                Member::Key(key) => JsonPathSegment::Key(key),
                Member::Index(index) => JsonPathSegment::Index(*index),
            })
            .collect();
        let width = match (self.width)(&segments) {
            Some(Width::Double) if number.is_finite() => return,
            Some(width) => width,
            None => return,
        };
        let replacement = replacement(width, number < 0.0);
        let path = self
            .path
            .iter()
            .map(|member| match member {
                Member::Key(key) => Member::Key(key.clone()),
                Member::Index(index) => Member::Index(*index),
            })
            .collect();
        self.hits.push(Hit {
            edit: Edit {
                start,
                old: token.len(),
                new: replacement.len(),
            },
            replacement,
            path: Some(path),
        });
    }

    /// A value has ended: in an object it ends a member, whose key may be one the object
    /// has had; then the hits of the copy that is not kept lose their warning.
    fn value_ended(&mut self) {
        let to = self.hits.len();
        let Some(Frame::Object {
            members,
            member_from,
            first_kept,
        }) = self.frames.last_mut()
        else {
            return;
        };
        let (Some(from), Some(Member::Key(key))) = (member_from.take(), self.path.pop()) else {
            return;
        };
        let unkept = match members.entry(key) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(from..to);
                return;
            }
            std::collections::hash_map::Entry::Occupied(_) if *first_kept => from..to,
            std::collections::hash_map::Entry::Occupied(mut entry) => entry.insert(from..to),
        };
        for hit in self.hits.get_mut(unkept).into_iter().flatten() {
            hit.path = None;
        }
    }
}

/// The largest finite number of the sign and width, as the shortest text that reads back to
/// it.
fn replacement(width: Width, negative: bool) -> &'static str {
    match (width, negative) {
        // It reads as an `f64` a little above `f32::MAX` and is still that float.
        (Width::Float, false) => "3.4028235e38",
        (Width::Float, true) => "-3.4028235e38",
        (Width::Double, false) => "1.7976931348623157e308",
        (Width::Double, true) => "-1.7976931348623157e308",
    }
}

/// Whether a number token of `text` reads as an infinity: one walk of the tokens, strings
/// and comments passed over. A text the reader refuses (a bad escape) has none to clamp.
fn holds_beyond_double(text: &str) -> bool {
    first_beyond_double(text).is_some()
}

/// Where the first number token of `text` that reads as an infinity is, as `start..end`
/// ([`holds_beyond_double`]'s walk). `None` when there is none, and for a text the reader
/// refuses before one.
pub(crate) fn first_beyond_double(text: &str) -> Option<std::ops::Range<usize>> {
    let bytes = text.as_bytes();
    let mut at = 0;
    while let Some(&byte) = bytes.get(at) {
        match byte {
            b'"' => match string_end(bytes, at) {
                Ok((end, _)) => at = end,
                Err(_) => return None,
            },
            b'/' => at = skip_trivia(bytes, at, true).max(at + 1),
            b'-' | b'0'..=b'9' => {
                let end = number_end(bytes, at).max(at + 1);
                // The shortest spelling of a number beyond a double without an exponent has
                // 309 digits.
                let token = text.get(at..end).unwrap_or_default();
                if (token.len() >= 309 || token.contains(['e', 'E']))
                    && token.parse::<f64>().is_ok_and(f64::is_infinite)
                {
                    return Some(at..end);
                }
                at = end;
            }
            _ => at += 1,
        }
    }
    None
}

/// The numbers of `value` (at `at`, from the root) beyond what holds them, in the order of
/// the value: a walk of the JSON, which asks `width` only for a number beyond `f32`.
fn wide_numbers<'v>(
    value: &'v Value,
    at: &mut Vec<JsonPathSegment<'v>>,
    width: &dyn Fn(&[JsonPathSegment<'_>]) -> Option<Width>,
    found: &mut Vec<Wide<'v>>,
) {
    if let Some(number) = value.as_number() {
        let number = number.to_f64_lossy();
        #[allow(
            clippy::cast_possible_truncation,
            reason = "the nearest f32 is the question"
        )]
        let nearest = number as f32;
        if nearest.is_finite() {
            return;
        }
        let width = match width(at) {
            Some(Width::Double) if number.is_finite() => None,
            width => width,
        };
        if let Some(width) = width {
            found.push(Wide {
                path: at.clone(),
                width,
                negative: number < 0.0,
            });
        }
    } else if let Some(members) = value.as_object() {
        for (key, member) in members {
            at.push(JsonPathSegment::Key(key.as_str()));
            wide_numbers(member, at, width, found);
            at.pop();
        }
    } else if let Some(elements) = value.as_array() {
        for (index, element) in elements.iter().enumerate() {
            at.push(JsonPathSegment::Index(index));
            wide_numbers(element, at, width, found);
            at.pop();
        }
    }
}

/// The members of an object or the elements of an array, each as its text.
enum Members<'a> {
    Object(IndexMap<String, RawJson<'a>>),
    Array(Vec<RawJson<'a>>),
}

/// Finds the number tokens of `wide` (numbers below the value of `slice`, which all share
/// their first `level` segments, in the order of the value) in the text as written, into
/// `tokens` from `first` on. An object or array is split once for all the numbers below it.
fn locate<'a>(
    slice: JsonSlice<'a>,
    wide: &[Wide<'_>],
    level: usize,
    first: usize,
    tokens: &mut [Option<JsonSlice<'a>>],
) {
    if let [only] = wide
        && only.path.len() == level
    {
        // The value's text, without anything around it.
        let before = slice.text.len() - slice.text.trim_start().len();
        let token = slice.text.trim();
        if token.parse::<f64>().is_ok()
            && let Some(place) = tokens.get_mut(first)
        {
            *place = Some(JsonSlice {
                text: token,
                offset: slice.offset + before,
            });
        }
        return;
    }
    let members = match wide.first().and_then(|number| number.segment(level)) {
        Some(JsonPathSegment::Key(_)) => parse_at::<IndexMap<String, RawJson<'a>>>(slice)
            .ok()
            .map(Members::Object),
        Some(JsonPathSegment::Index(_)) => {
            parse_at::<Vec<RawJson<'a>>>(slice).ok().map(Members::Array)
        }
        None => None,
    };
    let Some(members) = members else { return };
    let mut start = 0;
    while start < wide.len() {
        let segment = wide.get(start).and_then(|number| number.segment(level));
        let mut end = start + 1;
        while wide
            .get(end)
            .is_some_and(|next| next.segment(level) == segment)
        {
            end += 1;
        }
        let raw = match (&members, segment) {
            (Members::Object(members), Some(JsonPathSegment::Key(key))) => members.get(key),
            (Members::Array(elements), Some(JsonPathSegment::Index(index))) => elements.get(index),
            _ => None,
        };
        if let Some(child) = raw.and_then(|raw| slice.nested(raw))
            && let Some(below) = wide.get(start..end)
        {
            locate(child, below, level + 1, first + start, tokens);
        }
        start = end;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Location, Phase, Severity};
    use crate::test_cost::{assert_linear_timed_at, timed};

    fn path(members: &[&str]) -> JsonPath {
        let mut path = JsonPath::new();
        for member in members {
            path.push(member);
        }
        path
    }

    fn finding(code: DiagnosticCode, severity: Severity, at: &[&str]) -> Diagnostic {
        Diagnostic::new(
            code,
            severity,
            Phase::Validate,
            Location::new(None, None, path(at), None),
        )
    }

    /// The clamp's warning at `at`.
    fn clamp_warning(at: &[&str]) -> Diagnostic {
        beyond_f32_warning(path(at), Some(Span { offset: 7, len: 5 }))
    }

    /// Whether the clamp's warning at `a/n` is left out beside the one finding `other`.
    fn left_out_beside(other: Diagnostic) -> bool {
        let mut d = Diagnostics::default();
        d.push(other);
        let mut beyond = Diagnostics::default();
        beyond.push(clamp_warning(&["a", "n"]));
        spoken_for(&d, &beyond) == [true]
    }

    #[test]
    fn a_finding_about_the_value_at_the_number_speaks_for_it() {
        let wrong_type = DiagnosticCode::InvalidType { text: None };
        assert!(left_out_beside(finding(
            wrong_type.clone(),
            Severity::Warning,
            &["a", "n"]
        )));
        // The member's own clamp, the very finding the clamp of the text would give.
        assert!(left_out_beside(clamp_warning(&["a", "n"])));
        // Another number, or a member below the number, is another place.
        assert!(!left_out_beside(finding(
            wrong_type.clone(),
            Severity::Warning,
            &["a", "m"]
        )));
        assert!(!left_out_beside(finding(
            wrong_type,
            Severity::Error,
            &["a", "n", "x"]
        )));
    }

    #[test]
    fn a_finding_that_names_an_unknown_member_does_not_speak_for_its_number() {
        for code in [
            DiagnosticCode::UnrecognizedField { text: None },
            DiagnosticCode::UnrecognizedComponent { text: None },
            DiagnosticCode::UnknownSound { text: None },
        ] {
            assert!(
                !left_out_beside(finding(code.clone(), Severity::Warning, &["a", "n"])),
                "{code:?}"
            );
        }
    }

    /// A kept error around the number refuses nothing: the member is held as written
    /// and the number in it is the clamp's to tell.
    #[test]
    fn a_kept_error_around_the_number_does_not_speak_for_it() {
        let missing = DiagnosticCode::MissingRequiredField { text: None };
        assert!(!left_out_beside(
            finding(missing, Severity::Error, &["a"]).into_kept()
        ));
    }

    #[test]
    fn a_member_refused_around_the_number_speaks_for_it_and_the_root_does_not() {
        let invalid = DiagnosticCode::InvalidValue { text: None };
        assert!(left_out_beside(finding(
            invalid.clone(),
            Severity::Error,
            &["a"]
        )));
        // A warning around the number refuses nothing (a legacy block's "Duplicate
        // component" is at the component it keeps); an error at the root is the file's.
        assert!(!left_out_beside(finding(
            DiagnosticCode::DroppedComponent { text: None },
            Severity::Warning,
            &["a"]
        )));
        assert!(!left_out_beside(finding(
            invalid.clone(),
            Severity::Warning,
            &["a"]
        )));
        assert!(!left_out_beside(finding(invalid, Severity::Error, &[])));
    }

    #[test]
    fn a_finding_of_another_file_does_not_speak_for_the_number() {
        use crate::pack_identity::PackPath;
        let mut theirs = finding(
            DiagnosticCode::InvalidType { text: None },
            Severity::Warning,
            &["a", "n"],
        );
        theirs.location.path = Some(PackPath::from_static("textures/a.json"));
        let mut ours = clamp_warning(&["a", "n"]);
        ours.location.path = Some(PackPath::from_static("textures/b.json"));
        let mut d = Diagnostics::default();
        d.push(theirs);
        let mut beyond = Diagnostics::default();
        beyond.push(ours.clone());
        assert_eq!(spoken_for(&d, &beyond), [false]);
        ours.location.path = Some(PackPath::from_static("textures/a.json"));
        let mut beyond = Diagnostics::default();
        beyond.push(ours);
        assert_eq!(spoken_for(&d, &beyond), [true]);
    }

    #[test]
    fn a_later_finding_at_the_number_takes_the_clamps_warning_back() {
        let range = finding(
            DiagnosticCode::InvalidType { text: None },
            Severity::Warning,
            &["a", "n"],
        );
        let unknown = finding(
            DiagnosticCode::UnrecognizedField { text: None },
            Severity::Warning,
            &["b"],
        );
        let mut clamp = Diagnostics::default();
        clamp.push(clamp_warning(&["a", "n"]));
        clamp.push(clamp_warning(&["b"]));
        clamp.push(clamp_warning(&["c"]));
        let mut d = Diagnostics::default();
        d.push(unknown.clone());
        d.extend(clamp.clone());
        d.push(range.clone());
        // A member that gives the clamp's own finding a second time has said it once.
        d.push(clamp_warning(&["c"]));
        drop_spoken_for(&mut d, &clamp);
        let left: Vec<Diagnostic> = d.into_iter().collect();
        assert_eq!(
            left,
            [unknown, clamp_warning(&["b"]), range, clamp_warning(&["c"])]
        );
    }

    #[test]
    fn telling_each_number_once_is_linear_in_the_findings() {
        use crate::test_cost::assert_linear_at;
        // `n` clamped numbers, each with a finding of its member, and `n` findings elsewhere.
        assert_linear_at(
            "core drop_spoken_for",
            20_000,
            |n| {
                let mut d = Diagnostics::default();
                let mut clamp = Diagnostics::default();
                for i in 0..n {
                    let member = format!("m{i}");
                    let at = ["list", member.as_str(), "n"];
                    clamp.push(clamp_warning(&at));
                    d.push(clamp_warning(&at));
                    d.push(finding(
                        DiagnosticCode::InvalidType { text: None },
                        Severity::Warning,
                        &at,
                    ));
                    d.push(finding(
                        DiagnosticCode::UnrecognizedField { text: None },
                        Severity::Warning,
                        &["other", member.as_str()],
                    ));
                }
                (d, clamp)
            },
            |(d, clamp)| {
                let mut d = d.clone();
                drop_spoken_for(&mut d, clamp);
                assert_eq!(d.len(), 2 * clamp.len());
            },
        );
    }

    #[test]
    fn clamping_the_numbers_beyond_their_width_is_linear_in_the_numbers() {
        // `n` objects, each with a number beyond `f32`, beside as many numbers no double
        // holds in an array kept as JSON.
        assert_linear_timed_at("core clamp_text", 2_500, |n| {
            let controllers: Vec<String> = (0..n)
                .map(|i| format!(r#"{{"controller.render.c{i}": 1e39}}"#))
                .collect();
            let versions = vec!["-1e999"; n];
            let text = format!(
                r#"{{"kept": [{}], "floats": [{}]}}"#,
                versions.join(", "),
                controllers.join(", ")
            );
            let json: Value = crate::json::from_str(&text).expect("test JSON");
            let width = |path: &[JsonPathSegment<'_>]| match path.first() {
                Some(JsonPathSegment::Key("kept")) => Some(Width::Double),
                _ => Some(Width::Float),
            };
            let mut found = Diagnostics::default();
            let (clamped, took) = timed(|| {
                clamp_text(
                    JsonSlice::new(&text),
                    &json,
                    &width,
                    &JsonPath::new(),
                    &mut found,
                )
                .expect("numbers to clamp")
            });
            assert_eq!(found.len(), 2 * n);
            assert_eq!(clamped.text.matches("3.4028235e38").count(), n);
            assert_eq!(clamped.text.matches("-1.7976931348623157e308").count(), n);
            took
        });
    }

    /// The clamped text and the warnings as `(path, token under the span)`.
    fn clamped(text: &str, first_kept_in: &[&str]) -> (String, Vec<(String, String)>) {
        let mut found = Diagnostics::default();
        let clamped = clamp_beyond_double_first_kept(
            JsonSlice::new(text),
            first_kept_in,
            &JsonPath::new(),
            &mut found,
        )
        .expect("a number to clamp");
        let told = found
            .iter()
            .map(|finding| {
                let span = finding.location.span.expect("the token's span");
                let start = span.offset as usize;
                (
                    finding.location.json_path.to_string(),
                    text[start..start + span.len as usize].to_owned(),
                )
            })
            .collect();
        (clamped.text, told)
    }

    fn told(path: &str, token: &str) -> (String, String) {
        (path.to_owned(), token.to_owned())
    }

    #[test]
    fn every_copy_of_a_member_written_twice_is_clamped_and_the_last_is_told() {
        let (text, found) = clamped(r#"{"a": 1e999, "b": [0, -1e999], "a": 2e999}"#, &[]);
        assert_eq!(
            text,
            r#"{"a": 1.7976931348623157e308, "b": [0, -1.7976931348623157e308], "a": 1.7976931348623157e308}"#
        );
        assert_eq!(found, [told("b/1", "-1e999"), told("a", "2e999")]);
        // The last copy holds no such number: the first is clamped without a word.
        let (text, found) = clamped(r#"{"a": {"n": 1e999}, "a": {"n": 0}}"#, &[]);
        assert_eq!(
            text,
            r#"{"a": {"n": 1.7976931348623157e308}, "a": {"n": 0}}"#
        );
        assert_eq!(found, []);
    }

    #[test]
    fn where_the_family_keeps_the_first_copy_the_first_is_told() {
        let keep = &["components"];
        let (text, found) = clamped(
            r#"{"components": {"x:c": {"n": 1e999}, "x:c": {"n": -1e999}, "x:d": 3e999}}"#,
            keep,
        );
        assert_eq!(
            text,
            r#"{"components": {"x:c": {"n": 1.7976931348623157e308}, "x:c": {"n": -1.7976931348623157e308}, "x:d": 1.7976931348623157e308}}"#
        );
        assert_eq!(
            found,
            [
                told("components/x:c/n", "1e999"),
                told("components/x:d", "3e999")
            ]
        );
        // Only the members of that object: a member written twice inside a component, and
        // one beside `components`, keep their last copy.
        let (_, found) = clamped(
            r#"{"components": {"x:c": {"n": 1e999, "n": 2e999}}, "z": 3e999, "z": 4e999}"#,
            keep,
        );
        assert_eq!(
            found,
            [told("components/x:c/n", "2e999"), told("z", "4e999")]
        );
    }

    #[test]
    fn the_walk_names_keys_as_they_read_and_stops_with_the_root_value() {
        let (text, found) = clamped(
            "{ /* 1e999 */ \"a\\u0062\": [\"1e999\", {\"n\": 1e999}], // 1e999\n \"c\": true } 2e999",
            &[],
        );
        assert_eq!(found, [told("ab/1/n", "1e999")]);
        assert!(text.ends_with("} 2e999"), "{text}");
        // A text that is not well-formed is the parser's to report.
        let mut d = Diagnostics::default();
        for text in ["{\"a\": [1e999}", "[1e999", "{\"a\" 1e999]"] {
            let none = clamp_beyond_double(JsonSlice::new(text), &JsonPath::new(), &mut d);
            assert!(none.is_none(), "{text}");
        }
    }

    /// `n` members, each an object with a number, an array and a string.
    fn document(n: usize, number: &str) -> String {
        let members: Vec<String> = (0..n)
            .map(|i| {
                format!(
                    r#""member_{i}": {{"n": {number}, "list": [1, 2.5, {number}], "s": "1e999"}}"#
                )
            })
            .collect();
        format!("{{{}}}", members.join(", "))
    }

    #[test]
    fn clamping_the_numbers_no_double_holds_is_linear_in_the_numbers() {
        assert_linear_timed_at("core clamp_beyond_double", 5_000, |n| {
            let text = document(n, "-1e999");
            let mut found = Diagnostics::default();
            let (clamped, took) = timed(|| {
                clamp_beyond_double(JsonSlice::new(&text), &JsonPath::new(), &mut found)
                    .expect("numbers to clamp")
            });
            assert_eq!(found.len(), 2 * n);
            assert_eq!(
                clamped.text.matches("-1.7976931348623157e308").count(),
                2 * n
            );
            took
        });
    }

    #[test]
    fn a_text_without_a_number_beyond_a_double_costs_one_walk_of_its_tokens() {
        // The numbers have exponents, so each token is parsed to see that a double holds it;
        // no value is built and nothing is replaced.
        assert_linear_timed_at("core clamp_beyond_double, nothing to clamp", 20_000, |n| {
            let text = document(n, "1e300");
            let mut found = Diagnostics::default();
            let (clamped, took) =
                timed(|| clamp_beyond_double(JsonSlice::new(&text), &JsonPath::new(), &mut found));
            assert!(clamped.is_none() && found.is_empty());
            took
        });
    }
}
