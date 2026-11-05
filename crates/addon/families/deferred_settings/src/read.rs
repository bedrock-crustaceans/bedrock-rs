//! Reading a settings payload from its slice of the text: one object level at a time (its
//! members as raw slices, so every finding has the place of its own value), the scalar and
//! composite value forms with the game's texts, and the unknown members kept for write-back.
//!
//! The text is core's, in which a member written twice has its last value only.

use facet_json::RawJson;
use indexmap::IndexMap;

use super::value::{Description, Keyframe, Keyframes, Rgb8, Rgba8, Vec3};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity, Span, VanillaText,
};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::{Id, Kind};
use bedrock_addon_core::json::{JsonSlice, ScalarShape, Value, parse_at};
use bedrock_addon_core::version::FormatVersion;

/// The game's text for a value that is no number.
pub(crate) const MUST_BE_NUMBER: &str = "the input must be a number";
/// The game's text for a colour without alpha that cannot be read.
pub(crate) const RGB_TEXT: &str = "Expected [r, g, b], with values in the 0-255 range, or a hex value with 6 characters (e.g. RGB as #FFFFFF).";
/// The game's text for a colour with alpha that cannot be read.
pub(crate) const RGBA_TEXT: &str = "Expected [r, g, b, a], with values in the 0-255 range, or a hex value with 8 characters (e.g. ARGB as #FFFFFFFF).";
/// The game's text for keyframes of the wrong JSON type.
pub(crate) const KEYFRAMES_TEXT: &str = "Expected keyframes.";
/// The game's text for a triple of numbers that cannot be read.
pub(crate) const VEC3_TEXT: &str = "Expected [x, y, z].";
/// The game's text for a keyframe key that is no number; the placeholder is the name of the
/// target type.
pub(crate) const KEY_TEMPLATE: &str = "String-to-{} conversion failed";

/// The read failed; its reason is in the diagnostics.
///
/// A marker only: it means the finding was already pushed, and it carries nothing else.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Failed;

/// The outcome of reading one value.
pub(crate) type Read<T> = Result<T, Failed>;

/// A numeric constraint, both ends included.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Range {
    /// The lowest accepted value.
    pub(crate) min: f32,
    /// The highest accepted value.
    pub(crate) max: f32,
}

impl Range {
    /// `[min, max]`.
    pub(crate) const fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    /// Whether `value` lies within the range.
    pub(crate) fn contains(self, value: f32) -> bool {
        value >= self.min && value <= self.max
    }

    /// The game's text for a value outside the range.
    pub(crate) fn text(self) -> VanillaText {
        bedrock_addon_core::diagnostic::cereal::NumericRange::inclusive(
            f64::from(self.min),
            f64::from(self.max),
        )
        .text()
    }
}

/// A fixed vanilla text.
#[allow(
    clippy::unnecessary_wraps,
    reason = "the `text` member of every code is an `Option`"
)]
pub(crate) fn fixed(text: &'static str) -> Option<VanillaText> {
    Some(VanillaText::Fixed(text))
}

/// One JSON value of the document: its text and where it is.
#[derive(Clone, Debug)]
pub(crate) struct Node<'a> {
    /// The value's text, with its offset in the file.
    pub(crate) slice: JsonSlice<'a>,
    /// The value's path in the file.
    pub(crate) at: JsonPath,
}

impl<'a> Node<'a> {
    /// The value's place in the file.
    fn span(&self) -> Span {
        let narrow = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
        Span {
            offset: narrow(self.slice.offset),
            len: narrow(self.slice.text.len()),
        }
    }

    /// A finding about this value.
    pub(crate) fn finding(&self, code: DiagnosticCode, severity: Severity) -> Diagnostic {
        Diagnostic::error(code)
            .severity(severity)
            .at(self.at.clone())
            .span(self.span())
    }

    /// Reports an error about this value.
    pub(crate) fn error(&self, code: DiagnosticCode, d: &mut Diagnostics) -> Failed {
        d.push(self.finding(code, Severity::Error));
        Failed
    }

    /// The member or element `raw` of this value, at `at`.
    fn child(&self, raw: &RawJson<'a>, at: JsonPath) -> Option<Node<'a>> {
        Some(Node {
            slice: self.slice.nested(raw)?,
            at,
        })
    }

    /// The first byte of the value that is neither white space nor inside a comment.
    fn first_token(&self) -> Option<u8> {
        let mut rest = self.slice.text.trim_start();
        loop {
            if let Some(comment) = rest.strip_prefix("//") {
                rest = comment.split_once('\n').map_or("", |(_, after)| after);
            } else if let Some(comment) = rest.strip_prefix("/*") {
                rest = comment.split_once("*/").map_or("", |(_, after)| after);
            } else {
                return rest.bytes().next();
            }
            rest = rest.trim_start();
        }
    }

    /// The value is a JSON `null`, which reads as an absent member (core's rule).
    fn is_null(&self) -> bool {
        self.first_token() == Some(b'n')
            && matches!(parse_at::<ScalarShape>(self.slice), Ok(ScalarShape::Null))
    }

    /// The value as a scalar; `None` for an object, an array or text that is no value.
    fn scalar(&self) -> Option<ScalarShape> {
        match self.first_token()? {
            b'{' | b'[' => None,
            _ => parse_at::<ScalarShape>(self.slice).ok(),
        }
    }
}

/// The members of one object, each a raw slice, taken out one by one; what is left is unknown
/// to the struct.
pub(crate) struct Members<'a> {
    node: Node<'a>,
    map: IndexMap<String, RawJson<'a>>,
}

/// The members of the object `node`; an `InvalidType` error ("expected an object") for any
/// other value.
pub(crate) fn members<'a>(node: &Node<'a>, d: &mut Diagnostics) -> Read<Members<'a>> {
    if node.first_token() != Some(b'{') {
        return Err(node.error(bedrock_addon_core::diagnostic::cereal::expected_object(), d));
    }
    match parse_at::<IndexMap<String, RawJson<'a>>>(node.slice) {
        Ok(map) => Ok(Members {
            node: node.clone(),
            map,
        }),
        Err(_) => Err(node.error(bedrock_addon_core::diagnostic::cereal::expected_object(), d)),
    }
}

impl<'a> Members<'a> {
    /// The path of the member `key`.
    fn path(&self, key: &str) -> JsonPath {
        self.node.at.child(key)
    }

    /// Takes the member `key` out; `None` when it is absent or `null`.
    fn take(&mut self, key: &str) -> Option<Node<'a>> {
        let raw = self.map.shift_remove(key)?;
        let node = self.node.child(&raw, self.path(key))?;
        (!node.is_null()).then_some(node)
    }

    /// The optional member `key`, read by `read`.
    pub(crate) fn opt<T>(
        &mut self,
        key: &str,
        d: &mut Diagnostics,
        read: impl FnOnce(&Node<'a>, &mut Diagnostics) -> Read<T>,
    ) -> Read<Option<T>> {
        match self.take(key) {
            Some(node) => read(&node, d).map(Some),
            None => Ok(None),
        }
    }

    /// The required member `key`, read by `read`; a `MissingRequiredField` error at the
    /// member's path when it is absent.
    pub(crate) fn req<T>(
        &mut self,
        key: &str,
        d: &mut Diagnostics,
        read: impl FnOnce(&Node<'a>, &mut Diagnostics) -> Read<T>,
    ) -> Read<T> {
        if let Some(node) = self.take(key) {
            return read(&node, d);
        }
        d.push(
            Diagnostic::error(DiagnosticCode::MissingRequiredField { text: None })
                .at(self.path(key))
                .span(self.node.span()),
        );
        Err(Failed)
    }

    /// The members left, in document order: `(key, value)`.
    pub(crate) fn rest(self) -> impl Iterator<Item = (String, Node<'a>)> {
        let Members { node, map } = self;
        map.into_iter().filter_map(move |(key, raw)| {
            let at = node.at.child(&key);
            let child = node.child(&raw, at)?;
            Some((key, child))
        })
    }

    /// Ends the object: every member left is unknown to the struct, an `UnrecognizedField`
    /// information (the game is silent about a file that loads) and kept. A member named in
    /// `later` is declared by a newer struct: the finding names the `format_version` it needs.
    /// A number of a kept member that is beyond a double is replaced (`kept`).
    pub(crate) fn finish(
        self,
        later: &[(&'static str, FormatVersion)],
        d: &mut Diagnostics,
    ) -> UnknownMembers {
        self.finish_with(later, &[], d)
    }

    /// [`finish`](Self::finish) for a struct that also has members an older struct declared
    /// and this one no longer does: a member named in `gone` is unknown like any other, and
    /// its finding names the `format_version` from which it is no longer used.
    pub(crate) fn finish_with(
        self,
        later: &[(&'static str, FormatVersion)],
        gone: &[(&'static str, FormatVersion)],
        d: &mut Diagnostics,
    ) -> UnknownMembers {
        let hint = |template: &'static str, known: &[(&'static str, FormatVersion)], key: &str| {
            known
                .iter()
                .find(|(name, _)| *name == key)
                .map(|(name, version)| VanillaText::Formatted {
                    template,
                    args: vec![(*name).to_owned(), version.to_string()],
                })
        };
        let mut extra = UnknownMembers::default();
        for (key, node) in self.rest() {
            let text = hint("{} needs format_version {}", later, &key)
                .or_else(|| hint("{} is no longer used (format_version {})", gone, &key));
            // The game reads every deferred settings file leniently: an unknown member is no
            // error, and the parse log is printed only for an invalid document. The game is
            // silent about a file that loads, so the finding is an information.
            d.push(node.finding(DiagnosticCode::UnrecognizedField { text }, Severity::Info));
            if let Some(value) = kept(&node, d) {
                extra.insert(key, value);
            }
        }
        extra
    }
}

/// The unknown member `node` as it is kept for write-back: its JSON value. A number in it
/// beyond a double (`1e999`) has no value to keep and none the writer could write: it is kept
/// as the largest finite double of its sign, with an `InvalidValue` information at the number,
/// its own path and span (the clamp is the crate's own, for the write-back; core's walk does
/// the replacing).
fn kept(node: &Node<'_>, d: &mut Diagnostics) -> Option<Value> {
    let mut value = bedrock_addon_core::json::from_str_jsonc::<Value>(node.slice.text).ok()?;
    for clamped in bedrock_addon_core::json::clamp_non_finite(&mut value, Some(node.slice)) {
        let mut at = node.at.clone();
        at.append(&clamped.path);
        let finding = Diagnostic::info(DiagnosticCode::InvalidValue { text: None }).at(at);
        d.push(match clamped.span {
            Some(span) => finding.span(span),
            None => finding,
        });
    }
    Some(value)
}

/// A JSON number: its value and whether it was an integer token.
fn number(node: &Node<'_>, d: &mut Diagnostics) -> Read<(f64, bool)> {
    #[allow(
        clippy::cast_precision_loss,
        reason = "as the game reads an integer into a float"
    )]
    match node.scalar() {
        Some(ScalarShape::UInt(value)) => Ok((value as f64, true)),
        Some(ScalarShape::Int(value)) => Ok((value as f64, true)),
        Some(ScalarShape::Number(value)) => Ok((value, false)),
        _ => Err(node.error(
            DiagnosticCode::InvalidType {
                text: fixed(MUST_BE_NUMBER),
            },
            d,
        )),
    }
}

/// A number in `range`: an `InvalidValue` error with the game's range text outside it. A
/// number beyond `f32` is outside every range.
pub(crate) fn float_in(range: Range, node: &Node<'_>, d: &mut Diagnostics) -> Read<f32> {
    let (value, _) = number(node, d)?;
    #[allow(clippy::cast_possible_truncation, reason = "the game reads a float")]
    let narrow = value as f32;
    if range.contains(narrow) {
        Ok(narrow)
    } else {
        Err(node.error(
            DiagnosticCode::InvalidValue {
                text: Some(range.text()),
            },
            d,
        ))
    }
}

/// A number without a constraint. One beyond `f32` is read as the largest finite float of its
/// sign with an `InvalidValue` information (the crate's own): it never loads as an infinity
/// the writer would refuse.
pub(crate) fn float(node: &Node<'_>, d: &mut Diagnostics) -> Read<f32> {
    let (value, _) = number(node, d)?;
    #[allow(clippy::cast_possible_truncation, reason = "the game reads a float")]
    let narrow = value as f32;
    if narrow.is_finite() {
        return Ok(narrow);
    }
    d.push(node.finding(DiagnosticCode::InvalidValue { text: None }, Severity::Info));
    Ok(if value < 0.0 { f32::MIN } else { f32::MAX })
}

/// An integer in `[min, max]`. A real token with no fraction is taken as the integer (it is
/// not known whether the game's integer reader takes `16.0`); one with a fraction is an
/// `InvalidValue` error.
pub(crate) fn integer_in(min: i32, max: i32, node: &Node<'_>, d: &mut Diagnostics) -> Read<i32> {
    let (value, _) = number(node, d)?;
    if value.fract() != 0.0 {
        return Err(node.error(DiagnosticCode::InvalidValue { text: None }, d));
    }
    if value < f64::from(min) || value > f64::from(max) {
        #[allow(clippy::cast_precision_loss, reason = "the bounds are small integers")]
        let range = Range::new(min as f32, max as f32);
        return Err(node.error(
            DiagnosticCode::InvalidValue {
                text: Some(range.text()),
            },
            d,
        ));
    }
    #[allow(clippy::cast_possible_truncation, reason = "within [min, max]")]
    Ok(value as i32)
}

/// `true` or `false`.
pub(crate) fn boolean(node: &Node<'_>, d: &mut Diagnostics) -> Read<bool> {
    match node.scalar() {
        Some(ScalarShape::Bool(value)) => Ok(value),
        _ => Err(node.error(DiagnosticCode::InvalidType { text: None }, d)),
    }
}

/// A string.
pub(crate) fn string(node: &Node<'_>, d: &mut Diagnostics) -> Read<String> {
    match node.scalar() {
        Some(ScalarShape::String(value)) => Ok(value),
        _ => Err(node.error(DiagnosticCode::InvalidType { text: None }, d)),
    }
}

/// One of the names of `table`: an `InvalidEnumValue` error for any other string, an
/// `InvalidType` error for a value that is no string.
pub(crate) fn choice<T: Copy>(
    table: &[(&str, T)],
    node: &Node<'_>,
    d: &mut Diagnostics,
) -> Read<T> {
    let name = string(node, d)?;
    match table.iter().find(|(known, _)| *known == name) {
        Some((_, value)) => Ok(*value),
        None => Err(node.error(DiagnosticCode::InvalidEnumValue { text: None }, d)),
    }
}

/// `description.identifier`: any string, since the game puts no constraint on it. One without
/// a namespace is a `NamespaceRule` info (the game accepts it, so the finding is the crate's
/// own).
pub(crate) fn identifier<K: Kind>(node: &Node<'_>, d: &mut Diagnostics) -> Read<Id<K>> {
    let text = string(node, d)?;
    if !text.contains(':') {
        d.push(node.finding(DiagnosticCode::NamespaceRule { text: None }, Severity::Info));
    }
    Ok(Id::parse(text))
}

/// A `description` object: `identifier` is required inside it.
pub(crate) fn description<K: Kind>(node: &Node<'_>, d: &mut Diagnostics) -> Read<Description<K>> {
    let mut m = members(node, d)?;
    let identifier = m.req("identifier", d, identifier::<K>);
    let extra = m.finish(&[], d);
    Ok(Description {
        identifier: identifier?,
        extra,
    })
}

/// The channels of a colour: `N` integers 0–255, or `#` and `2 N` hex digits (the bytes in
/// the order written). An array element outside 0–255 is read as the nearest bound with a
/// `ClampedValue` info (vanilla clamps silently); one with a fraction is an `InvalidValue`
/// error; anything else is an `InvalidType` error with `text`.
fn channels<const N: usize>(
    text: &'static str,
    node: &Node<'_>,
    d: &mut Diagnostics,
) -> Read<[u8; N]> {
    let wrong = |at: &Node<'_>, d: &mut Diagnostics| {
        at.error(DiagnosticCode::InvalidType { text: fixed(text) }, d)
    };
    let mut out = [0_u8; N];
    match node.first_token() {
        Some(b'"') => {
            let Some(ScalarShape::String(hex)) = node.scalar() else {
                return Err(wrong(node, d));
            };
            let digits = hex.strip_prefix('#').unwrap_or("");
            if digits.len() != 2 * N || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(wrong(node, d));
            }
            for (i, channel) in out.iter_mut().enumerate() {
                *channel = u8::from_str_radix(&digits[2 * i..2 * i + 2], 16).unwrap_or(0);
            }
            Ok(out)
        }
        Some(b'[') => {
            let Ok(elements) = parse_at::<Vec<RawJson<'_>>>(node.slice) else {
                return Err(wrong(node, d));
            };
            if elements.len() != N {
                return Err(wrong(node, d));
            }
            let mut failed = false;
            for (i, (raw, channel)) in elements.iter().zip(out.iter_mut()).enumerate() {
                let at = node.at.index(i);
                let Some(element) = node.child(raw, at) else {
                    return Err(wrong(node, d));
                };
                let value = match element.scalar() {
                    #[allow(clippy::cast_precision_loss, reason = "clamped to 0–255 below")]
                    Some(ScalarShape::UInt(value)) => value as f64,
                    #[allow(clippy::cast_precision_loss, reason = "clamped to 0–255 below")]
                    Some(ScalarShape::Int(value)) => value as f64,
                    Some(ScalarShape::Number(value)) => value,
                    _ => {
                        wrong(&element, d);
                        failed = true;
                        continue;
                    }
                };
                if value.fract() != 0.0 {
                    element.error(DiagnosticCode::InvalidValue { text: None }, d);
                    failed = true;
                    continue;
                }
                let clamped = value.clamp(0.0, 255.0);
                #[allow(clippy::float_cmp, reason = "exact: whether the clamp changed it")]
                if clamped != value {
                    d.push(
                        element
                            .finding(DiagnosticCode::ClampedValue { text: None }, Severity::Info),
                    );
                }
                #[allow(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "clamped to 0–255"
                )]
                {
                    *channel = clamped as u8;
                }
            }
            if failed { Err(Failed) } else { Ok(out) }
        }
        _ => Err(wrong(node, d)),
    }
}

/// A colour without alpha: `[r, g, b]` or `"#RRGGBB"`.
pub(crate) fn rgb(node: &Node<'_>, d: &mut Diagnostics) -> Read<Rgb8> {
    let [red, green, blue] = channels::<3>(RGB_TEXT, node, d)?;
    Ok(Rgb8 { red, green, blue })
}

/// A colour with alpha: `[r, g, b, a]` or `"#AARRGGBB"` (alpha first in the hex form).
pub(crate) fn rgba(node: &Node<'_>, d: &mut Diagnostics) -> Read<Rgba8> {
    let hex = node.first_token() == Some(b'"');
    let [first, second, third, fourth] = channels::<4>(RGBA_TEXT, node, d)?;
    Ok(if hex {
        Rgba8::new(second, third, fourth, first)
    } else {
        Rgba8::new(first, second, third, fourth)
    })
}

/// The value of the hexadecimal float that follows a `0x`: hexadecimal digits with an
/// optional point, then an optional `p`, sign and decimal digits, the power of two the digits
/// are scaled by. `None` when no hexadecimal digit follows the `0x`. The value is rounded
/// once, to the nearest `f32`; beyond the type it is an infinity or zero.
fn hex_float(bytes: &[u8]) -> Option<f32> {
    // The digits that fit, as an integer times 2^`exponent`; `sticky` is whether a digit that
    // did not fit was other than 0.
    let mut mantissa: u64 = 0;
    let mut exponent: i32 = 0;
    let mut sticky = false;
    let mut digits = 0_usize;
    let mut point = false;
    let mut next = 0;
    loop {
        match bytes.get(next) {
            Some(b'.') if !point => point = true,
            Some(&byte) if byte.is_ascii_hexdigit() => {
                digits += 1;
                let digit = u64::from(char::from(byte).to_digit(16).unwrap_or(0));
                if mantissa >> 40 == 0 {
                    mantissa = mantissa << 4 | digit;
                    exponent = exponent.saturating_sub(if point { 4 } else { 0 });
                } else {
                    sticky |= digit != 0;
                    exponent = exponent.saturating_add(if point { 0 } else { 4 });
                }
            }
            _ => break,
        }
        next += 1;
    }
    if digits == 0 {
        return None;
    }
    if matches!(bytes.get(next), Some(b'p' | b'P')) {
        let mut at = next + 1;
        let negative = bytes.get(at) == Some(&b'-');
        if matches!(bytes.get(at), Some(b'+' | b'-')) {
            at += 1;
        }
        // An exponent without a digit is no part of the number.
        let mut scale: Option<i32> = None;
        while let Some(byte) = bytes.get(at).filter(|byte| byte.is_ascii_digit()) {
            let digit = i32::from(byte - b'0');
            scale = Some(scale.unwrap_or(0).saturating_mul(10).saturating_add(digit));
            at += 1;
        }
        if let Some(scale) = scale {
            exponent = if negative {
                exponent.saturating_sub(scale)
            } else {
                exponent.saturating_add(scale)
            };
        }
    }
    if mantissa == 0 {
        return Some(0.0);
    }
    // At most 31 bits and the sticky bit below them: more than an `f32` rounds at, and exact
    // in a double.
    while mantissa >> 31 != 0 {
        sticky |= mantissa & 1 != 0;
        mantissa >>= 1;
        exponent = exponent.saturating_add(1);
    }
    mantissa = mantissa << 1 | u64::from(sticky);
    exponent = exponent.saturating_sub(1);
    // Under 2^33 times a power of two: beyond ±400 the value is past either end of `f32`,
    // and within it the product is exact.
    #[allow(clippy::cast_precision_loss, reason = "under 2^33: exact")]
    let wide = mantissa as f64 * 2.0_f64.powi(exponent.clamp(-400, 400));
    #[allow(clippy::cast_possible_truncation, reason = "the one rounding")]
    Some(wide as f32)
}

/// What C99's `strtof` reads from the start of `text` (the game reads a keyframe key the
/// same way): leading white space, a sign, then `inf` / `nan`, a hexadecimal float (`0x`,
/// hexadecimal digits with an optional point, an optional binary exponent `p±d`), or decimal
/// digits with an optional fraction and exponent. `None` when no character is consumed.
pub(crate) fn strtof(text: &str) -> Option<f32> {
    let text = text.trim_start_matches(|c: char| c.is_ascii_whitespace() || c == '\u{b}');
    let bytes = text.as_bytes();
    let sign = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let negative = bytes.first() == Some(&b'-');
    if bytes.get(sign) == Some(&b'0') && matches!(bytes.get(sign + 1), Some(b'x' | b'X')) {
        // Without a hexadecimal digit the number is the `0` before the `x`.
        if let Some(value) = hex_float(&bytes[sign + 2..]) {
            return Some(if negative { -value } else { value });
        }
    }
    let word = |name: &str| {
        bytes
            .get(sign..sign + name.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(name.as_bytes()))
    };
    if word("inf") {
        return Some(if negative {
            f32::NEG_INFINITY
        } else {
            f32::INFINITY
        });
    }
    if word("nan") {
        return Some(f32::NAN);
    }
    let digits = |from: usize| {
        bytes[from..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count()
    };
    let whole = digits(sign);
    let mut end = sign + whole;
    let mut fraction = 0;
    if bytes.get(end) == Some(&b'.') {
        fraction = digits(end + 1);
        if whole + fraction > 0 {
            end += 1 + fraction;
        }
    }
    if whole + fraction == 0 {
        return None;
    }
    if matches!(bytes.get(end), Some(b'e' | b'E')) {
        let mut exponent = end + 1;
        if matches!(bytes.get(exponent), Some(b'+' | b'-')) {
            exponent += 1;
        }
        let count = digits(exponent);
        if count > 0 {
            end = exponent + count;
        }
    }
    text[..end].parse().ok()
}

/// A keyframed value: a single value read by `value`, or an object of at least one keyframe
/// whose keys are times.
///
/// - A key `strtof` reads nothing from is an `InvalidType` error with the game's text (the
///   crate skips the file; the game logs the text, reads the key as 0 and loads).
/// - A key outside `[0, 1]` (or NaN) is dropped with a `KeyframeOutOfDomain` information (the
///   game drops it silently).
/// - An empty object is an `InvalidValue` error (the game requires at least one keyframe); so
///   is an object all of whose keys were dropped (the crate's own rule).
/// - Of two keys of one time the later in the file stands, with an `InvalidValue`
///   information; the keyframe that does not stand is a `DroppedField` info at its own key
///   (the game keeps both). `-0` and `0` are one time, held as `0`.
/// - Any other JSON type is an `InvalidType` error, "Expected keyframes.".
pub(crate) fn keyframes<'a, T>(
    node: &Node<'a>,
    d: &mut Diagnostics,
    value: impl Fn(&Node<'a>, &mut Diagnostics) -> Read<T>,
) -> Read<Keyframes<T>> {
    if node.first_token() != Some(b'{') {
        // The single form: the value reader's own type error is the keyframe binder's.
        let mut own = Diagnostics::default();
        let read = value(node, &mut own);
        let mistyped = read.is_err()
            && own
                .iter()
                .any(|f| matches!(f.code, DiagnosticCode::InvalidType { .. }));
        if mistyped {
            return Err(node.error(
                DiagnosticCode::InvalidType {
                    text: fixed(KEYFRAMES_TEXT),
                },
                d,
            ));
        }
        for finding in own {
            d.push(finding);
        }
        return read.map(Keyframes::Constant);
    }
    let m = members(node, d)?;
    let mut frames = Vec::new();
    let mut failed = false;
    let mut any = false;
    for (key, child) in m.rest() {
        any = true;
        let read = value(&child, d);
        let Some(time) = strtof(&key) else {
            child.error(
                DiagnosticCode::InvalidType {
                    text: Some(VanillaText::Formatted {
                        template: KEY_TEMPLATE,
                        args: vec!["float".to_owned()],
                    }),
                },
                d,
            );
            failed = true;
            continue;
        };
        let Ok(value) = read else {
            failed = true;
            continue;
        };
        if !(0.0..=1.0).contains(&time) {
            // The game skips a key outside the domain without a line, so the finding is an
            // information.
            d.push(child.finding(DiagnosticCode::KeyframeOutOfDomain, Severity::Info));
            continue;
        }
        // `-0` is the time `0`.
        let time = time + 0.0;
        frames.push((Keyframe { time, value }, child));
    }
    if failed {
        return Err(Failed);
    }
    if !any || frames.is_empty() {
        return Err(node.error(DiagnosticCode::InvalidValue { text: None }, d));
    }
    Ok(Keyframes::Frames(standing(frames, d)))
}

/// The keyframes that stand, by time: of the keyframes of one time the last in `frames`
/// (which is in document order), with an `InvalidValue` information at it and a `DroppedField`
/// information at the one it replaces (the game keeps both keyframes without a word, so both
/// findings are the crate's own). The times are finite and none is `-0`.
pub(super) fn standing<T>(
    mut frames: Vec<(Keyframe<T>, Node<'_>)>,
    d: &mut Diagnostics,
) -> Vec<Keyframe<T>> {
    // Stable: keyframes of one time stay in document order.
    frames.sort_by(|a, b| a.0.time.total_cmp(&b.0.time));
    let mut kept: Vec<(Keyframe<T>, Node<'_>)> = Vec::with_capacity(frames.len());
    for (frame, child) in frames {
        #[allow(clippy::float_cmp, reason = "exact: two keys of one time")]
        if kept.last().is_some_and(|(last, _)| last.time == frame.time) {
            d.push(child.finding(DiagnosticCode::InvalidValue { text: None }, Severity::Info));
            if let Some((_, replaced)) = kept.pop() {
                d.push(
                    replaced.finding(DiagnosticCode::DroppedField { text: None }, Severity::Info),
                );
            }
        }
        kept.push((frame, child));
    }
    kept.into_iter().map(|(frame, _)| frame).collect()
}

/// A triple of numbers whose components lie in `range`: `[x, y, z]`, or an object of optional
/// `x`, `y`, `z` (0 when left out). An unknown member of the object form is an
/// `UnrecognizedField` finding and is not kept (the value is written back as the array).
#[allow(
    clippy::many_single_char_names,
    reason = "x, y and z are the members' names"
)]
pub(crate) fn vec3_in(range: Range, node: &Node<'_>, d: &mut Diagnostics) -> Read<Vec3> {
    let wrong = |d: &mut Diagnostics| {
        node.error(
            DiagnosticCode::InvalidType {
                text: fixed(VEC3_TEXT),
            },
            d,
        )
    };
    match node.first_token() {
        Some(b'[') => {
            let Ok(elements) = parse_at::<Vec<RawJson<'_>>>(node.slice) else {
                return Err(wrong(d));
            };
            if elements.len() != 3 {
                return Err(wrong(d));
            }
            let mut out = [0.0_f32; 3];
            let mut failed = false;
            for (i, (raw, component)) in elements.iter().zip(out.iter_mut()).enumerate() {
                let at = node.at.index(i);
                let Some(element) = node.child(raw, at) else {
                    return Err(wrong(d));
                };
                match float_in(range, &element, d) {
                    Ok(value) => *component = value,
                    Err(Failed) => failed = true,
                }
            }
            if failed {
                Err(Failed)
            } else {
                Ok(Vec3::new(out[0], out[1], out[2]))
            }
        }
        Some(b'{') => {
            let mut m = members(node, d)?;
            let x = m.opt("x", d, |n, d| float_in(range, n, d));
            let y = m.opt("y", d, |n, d| float_in(range, n, d));
            let z = m.opt("z", d, |n, d| float_in(range, n, d));
            m.finish(&[], d);
            Ok(Vec3::new(
                x?.unwrap_or(0.0),
                y?.unwrap_or(0.0),
                z?.unwrap_or(0.0),
            ))
        }
        _ => Err(wrong(d)),
    }
}

/// The payload a version struct parses, as a node at the payload's path.
pub(crate) fn payload<'a>(
    slice: JsonSlice<'a>,
    cx: &bedrock_addon_core::versioning::ParseCx<'_>,
) -> Node<'a> {
    Node {
        slice,
        at: cx.at.clone(),
    }
}

#[cfg(test)]
mod failed_tests {
    use super::*;

    /// The failure marker compares equal to itself, so a `Read` can be asserted on.
    #[test]
    fn a_failed_read_equals_a_failed_read() {
        let read: Read<u8> = Err(Failed);
        assert_eq!(read, Err(Failed));
    }
}
