//! Reading a settings payload: the profile the game's lenient reader has, and the value forms its
//! members take (a number in a range, a colour, a triple, keyframes by time) with the game's
//! texts. The documents' structs derive the rest of the read.

use bedrock_addon_core::diagnostic::cereal::{self, NumericRange};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, CodeDef, Diagnostic, Diagnostics, Kind as CodeKind, Severity, Span,
};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::{Id, Kind};
use bedrock_addon_core::json::kit::{
    Expected, MemberOrder, Members, Node, Profile, Read, Texts, UnknownAt, WrongKind,
};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::schema::{FromJson, UnknownRule};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{NullHandling, ParseCx, Scalars, read_payload_with};
use bedrock_addon_shared_shapes::color::hex_bytes;

use super::codes;
use super::value::{Keyframe, Keyframes, Rgb8, Rgba8, Vec3};

/// A numeric constraint, both ends included: core's `cereal::NumericRange`, with the `f32` ends the
/// game writes its constraints in ([`range`]).
pub(crate) type Range = NumericRange;

/// `[min, max]`, the `f32` ends widened exactly, so a value compares and a finding prints as
/// against the `f32`s.
#[allow(
    clippy::cast_lossless,
    reason = "`f64::from` is not a const fn; an f32 widens to f64 exactly"
)]
pub(crate) const fn range(min: f32, max: f32) -> Range {
    Range::inclusive(min as f64, max as f64)
}

/// The game's reader binds every member in the order it declares them, tells each failure, and
/// tells the members it does not know last.
const PROFILE: Profile = Profile::EVERY_FAILURE
    .with_members(MemberOrder::Declaration)
    .with_unknown(UnknownAt::Last);

/// The text of a value of the wrong JSON type: a number is asked for by its own line, an object
/// by cereal's.
fn wrong_kind(expected: Expected, _: &Value) -> Code {
    match expected {
        Expected::Number => Code::from(&codes::MUST_BE_NUMBER),
        Expected::Object => cereal::expected_object(),
        _ => Code::from(&core_codes::INVALID_TYPE),
    }
}

/// `read` as the game's reader of a payload reads: every member told where it is met, a `null`
/// member absent, an unknown member an information (the game loads a file with one) kept for
/// write-back.
pub(crate) fn lenient(read: Read<'_>) -> Read<'_> {
    read.profile(PROFILE)
        .texts(Texts {
            wrong_kind: Some(WrongKind(wrong_kind)),
            ..Texts::CODES
        })
        .null(NullHandling::Own)
        .scalars(Scalars::Strict)
        .unknown_severity(Severity::Info)
        .clamp_kept()
}

/// Reads the payload `payload` of a version struct `T` ([`lenient`]).
pub(crate) fn payload<T: FromJson>(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<T> {
    read_payload_with(payload, cx, d, lenient)
}

/// [`payload`] with the unknown members `table` names told as the members of another format
/// version ([`hints`]).
pub(crate) fn payload_hinted<T: FromJson>(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
    table: &[Hint],
) -> Option<T> {
    let from = d.len();
    let value = self::payload(payload, cx, d);
    hints(d, from, cx, table);
    value
}

/// The member of a newer or older struct that the struct read does not know: an unknown member
/// whose finding names the `format_version` it is declared from (`NeedsFormatVersion`) or is
/// no longer used from (`NoLongerUsed`).
pub(crate) type Hint = (&'static str, &'static CodeDef<2>, FormatVersion);

/// The member `path` of a struct is declared from the format `version` on.
pub(crate) const fn later(path: &'static str, version: FormatVersion) -> Hint {
    (path, &codes::NEEDS_FORMAT_VERSION, version)
}

/// The member `path` of a struct is no longer used from the format `version` on.
pub(crate) const fn gone(path: &'static str, version: FormatVersion) -> Hint {
    (path, &codes::NO_LONGER_USED, version)
}

/// Gives the unknown members of `d` from `from` on that `table` names (by their path below the
/// payload, `waves/sampleWidth`) the finding of the format they belong to. The game is silent
/// about a file that loads, so the finding is the crate's own, an information.
pub(crate) fn hints(d: &mut Diagnostics, from: usize, cx: &ParseCx<'_>, table: &[Hint]) {
    if table.is_empty() {
        return;
    }
    let told = std::mem::take(d);
    for (index, mut finding) in told.into_iter().enumerate() {
        if index >= from && finding.code.is(&core_codes::UNRECOGNIZED_FIELD) {
            let hint = table.iter().find(|(path, ..)| {
                let mut at = cx.at.clone();
                path.split('/').for_each(|step| at.push(step));
                at == finding.location.json_path
            });
            if let Some((path, def, version)) = hint {
                let name = path.rsplit('/').next().unwrap_or(path);
                finding.code = def.with([name.to_owned(), version.to_string()]);
            }
        }
        d.push(finding);
    }
}

/// [`payload_hinted`] of a struct that requires the member `member` of the payload, which the game
/// asks for first: a missing one (or a `null`) is told before every other finding of the read,
/// whether or not the rest reads, and rejects the payload.
pub(crate) fn payload_requiring<T: FromJson>(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
    table: &[Hint],
    member: &str,
) -> Option<T> {
    let from = d.len();
    let value = payload_hinted(payload, cx, d, table);
    let lacking = bedrock_addon_core::json::from_str_jsonc(payload.text)
        .ok()
        .and_then(|json| {
            json.as_object()
                .map(|object| object.get(member).is_none_or(Value::is_null))
        })
        .unwrap_or(false);
    if !lacking {
        return value;
    }
    let narrow = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    let missing = Diagnostic::of(&core_codes::MISSING_REQUIRED_FIELD)
        .at(cx.at.child(member))
        .span(Span {
            offset: narrow(payload.offset),
            len: narrow(payload.text.len()),
        });
    let told = std::mem::take(d);
    for (index, finding) in told.into_iter().enumerate() {
        if index == from {
            d.push(missing.clone());
        }
        d.push(finding);
    }
    if d.len() == from {
        d.push(missing);
    }
    None
}

/// A finding of the crate's own about the value at `node`, an information.
pub(crate) fn info(node: &Node<'_>, code: Code) -> Diagnostic {
    let mut finding = Diagnostic::of(code)
        .severity(Severity::Info)
        .at(node.path().clone());
    finding.location.span = node.span();
    finding
}

/// A number in `range` (`schema::read_with` of a ranged member): an `InvalidValue` error with the
/// game's range text outside it. A number beyond `f32` is outside every range.
pub(crate) fn float_in(range: Range) -> impl Fn(&Node<'_>, &mut Read<'_>) -> Option<f32> {
    move |node, read| {
        let value = read.number(node)?;
        #[allow(clippy::cast_possible_truncation, reason = "the game reads a float")]
        let narrow = value as f32;
        if range.contains(f64::from(narrow)) {
            return Some(narrow);
        }
        read.refuse(node, range.code());
        None
    }
}

/// A whole number in `[min, max]` (`schema::read_with` of a member of an integer type). A real
/// token with no fraction is taken as the integer (it is not known whether the game's integer
/// reader takes `16.0`); one with a fraction is an `InvalidValue` error.
pub(crate) fn whole_in<T>(bounds: (T, T)) -> impl Fn(&Node<'_>, &mut Read<'_>) -> Option<T>
where
    T: Copy + Into<f64> + TryFrom<i64>,
{
    move |node, read| {
        let (min, max) = (bounds.0.into(), bounds.1.into());
        let value = read.number(node)?;
        if value.fract() != 0.0 {
            read.refuse(node, Code::from(&core_codes::INVALID_VALUE));
            return None;
        }
        if value < min || value > max {
            read.refuse(node, Range::inclusive(min, max).code());
            return None;
        }
        #[allow(clippy::cast_possible_truncation, reason = "within [min, max]")]
        T::try_from(value as i64).ok()
    }
}

/// A member the game asks for, read by `value`: a `null` is no value, a missing member (told at
/// the member, over the `null`).
pub(crate) fn present<T>(
    value: impl Fn(&Node<'_>, &mut Read<'_>) -> Option<T>,
) -> impl Fn(&Node<'_>, &mut Read<'_>) -> Option<T> {
    move |node, read| {
        if node.value().is_null() {
            read.refuse(node, Code::from(&core_codes::MISSING_REQUIRED_FIELD));
            return None;
        }
        value(node, read)
    }
}

/// `description.identifier`: any string, since the game puts no constraint on it. One without a
/// namespace is a `NamespaceRule` info (the game accepts it, so the finding is the crate's own).
pub(crate) fn identifier<K: Kind>(node: &Node<'_>, read: &mut Read<'_>) -> Option<Id<K>> {
    let text = read.string(node)?;
    if !text.contains(':') {
        read.push(info(node, Code::from(&core_codes::NAMESPACE_RULE)));
    }
    Some(Id::parse(text))
}

/// The unknown members of `members` that `known` does not name: each told as the game's reader
/// tells it (an information, and one for each number of it beyond a double), and kept.
pub(crate) fn unknown(
    read: &mut Read<'_>,
    members: &Members<'_>,
    known: &[&str],
) -> UnknownMembers {
    let mut kept = UnknownMembers::default();
    for (key, member) in members.iter().filter(|(key, _)| !known.contains(key)) {
        read.unknown_member(&member, UnknownRule::Keep);
        if let Some(value) = Value::from_json(&member, read) {
            kept.insert(key.to_owned(), value);
        }
    }
    kept
}

/// The channels of a colour: `N` integers 0–255, or `#` and `2 N` hex digits (the bytes in
/// the order written). An array element outside 0–255 is read as the nearest bound with a
/// `ClampedValue` info (vanilla clamps silently); one with a fraction is an `InvalidValue`
/// error; anything else is an `InvalidType` error with `wrong_type`.
fn channels<const N: usize>(
    wrong_type: &'static CodeDef<0>,
    node: &Node<'_>,
    read: &mut Read<'_>,
) -> Option<[u8; N]> {
    if read.stopped() {
        return None;
    }
    let value = node.value();
    if let Some(hex) = value.as_string() {
        let bytes = hex_bytes::<N>(hex.as_str());
        if bytes.is_none() {
            read.refuse(node, Code::from(wrong_type));
        }
        return bytes;
    }
    if value.as_array().is_none_or(|elements| elements.len() != N) {
        read.refuse(node, Code::from(wrong_type));
        return None;
    }
    let elements = read.array(node)?;
    let mut out = [0_u8; N];
    let mut failed = false;
    for (index, channel) in out.iter_mut().enumerate() {
        let element = elements.element(index)?;
        let Some(number) = element.value().as_number() else {
            read.refuse(&element, Code::from(wrong_type));
            failed = true;
            continue;
        };
        let number = number.to_f64_lossy();
        if number.fract() != 0.0 {
            read.refuse(&element, Code::from(&core_codes::INVALID_VALUE));
            failed = true;
            continue;
        }
        let clamped = number.clamp(0.0, 255.0);
        #[allow(clippy::float_cmp, reason = "exact: whether the clamp changed it")]
        if clamped != number {
            read.push(info(&element, Code::from(&core_codes::CLAMPED_VALUE)));
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
    (!failed).then_some(out)
}

/// A colour without alpha: `[r, g, b]` or `"#RRGGBB"`.
impl FromJson for Rgb8 {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let [r, g, b] = channels::<3>(&codes::RGB, node, read)?;
        Some(Self::new(r, g, b))
    }
}

/// A colour with alpha: `[r, g, b, a]` or `"#AARRGGBB"` (alpha first in the hex form).
impl FromJson for Rgba8 {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let hex = node.value().is_string();
        let [first, second, third, fourth] = channels::<4>(&codes::RGBA, node, read)?;
        Some(if hex {
            Self::new(second, third, fourth, first)
        } else {
            Self::new(first, second, third, fourth)
        })
    }
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

/// Keyframed numbers in `range` (`schema::read_with` of a member).
pub(crate) fn keyframed_in(
    range: Range,
) -> impl Fn(&Node<'_>, &mut Read<'_>) -> Option<Keyframes<f32>> {
    keyframed(float_in(range))
}

/// A keyframed value, each value read by `value` (`schema::read_with` of a member): see
/// [`keyframes`].
pub(crate) fn keyframed<T>(
    value: impl Fn(&Node<'_>, &mut Read<'_>) -> Option<T>,
) -> impl Fn(&Node<'_>, &mut Read<'_>) -> Option<Keyframes<T>> {
    move |node, read| keyframes(node, read, &value)
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
pub(crate) fn keyframes<T>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    value: impl Fn(&Node<'_>, &mut Read<'_>) -> Option<T>,
) -> Option<Keyframes<T>> {
    if read.stopped() {
        return None;
    }
    if !node.value().is_object() {
        // The single form: the value reader's own type error is the keyframe binder's.
        let mut attempt = read.fork();
        let single = value(node, &mut attempt);
        // By kind: the value readers raise the family's and core's wrong-type definitions, the
        // only ones of the kind that can occur here.
        let mistyped = single.is_none()
            && attempt
                .findings_mut()
                .iter()
                .any(|found| found.kind() == CodeKind::WrongType);
        if mistyped {
            read.refuse(node, Code::from(&codes::KEYFRAMES));
            return None;
        }
        read.commit(attempt);
        return single.map(Keyframes::Single);
    }
    let members = read.object(node)?;
    let mut frames = Vec::new();
    let mut failed = false;
    let mut any = false;
    for (key, child) in members.iter() {
        any = true;
        let read_value = value(&child, read);
        let Some(time) = strtof(key) else {
            read.refuse(&child, codes::KEY_CONVERSION.with(["float"]));
            failed = true;
            continue;
        };
        let Some(read_value) = read_value else {
            failed = true;
            continue;
        };
        if !(0.0..=1.0).contains(&time) {
            // The game skips a key outside the domain without a line, so the finding is an
            // information.
            read.push(info(
                &child,
                Code::from(&core_codes::KEYFRAME_OUT_OF_DOMAIN),
            ));
            continue;
        }
        // `-0` is the time `0`.
        let time = time + 0.0;
        frames.push((
            Keyframe {
                time,
                value: read_value,
            },
            child,
        ));
    }
    if failed {
        return None;
    }
    if !any || frames.is_empty() {
        read.refuse(node, Code::from(&core_codes::INVALID_VALUE));
        return None;
    }
    Some(Keyframes::Keys(standing(frames, read)))
}

/// The keyframes that stand, by time: of the keyframes of one time the last in `frames`
/// (which is in document order), with an `InvalidValue` information at it and a `DroppedField`
/// information at the one it replaces (the game keeps both keyframes without a word, so both
/// findings are the crate's own). The times are finite and none is `-0`.
pub(super) fn standing<T>(
    mut frames: Vec<(Keyframe<T>, Node<'_>)>,
    read: &mut Read<'_>,
) -> Vec<Keyframe<T>> {
    // Stable: keyframes of one time stay in document order.
    frames.sort_by(|a, b| a.0.time.total_cmp(&b.0.time));
    let mut kept: Vec<(Keyframe<T>, Node<'_>)> = Vec::with_capacity(frames.len());
    for (frame, child) in frames {
        #[allow(clippy::float_cmp, reason = "exact: two keys of one time")]
        if kept.last().is_some_and(|(last, _)| last.time == frame.time) {
            read.push(info(&child, Code::from(&core_codes::INVALID_VALUE)));
            if let Some((_, replaced)) = kept.pop() {
                read.push(info(&replaced, Code::from(&core_codes::DROPPED_FIELD)));
            }
        }
        kept.push((frame, child));
    }
    kept.into_iter().map(|(frame, _)| frame).collect()
}

/// A triple of numbers whose components lie in `range` (`schema::read_with` of a member):
/// `[x, y, z]`, or an object of optional `x`, `y`, `z` (0 when left out). An unknown member of
/// the object form is an `UnrecognizedField` finding and is not kept (the value is written back
/// as the array).
pub(crate) fn vec3_in(range: Range) -> impl Fn(&Node<'_>, &mut Read<'_>) -> Option<Vec3> {
    move |node, read| {
        if read.stopped() {
            return None;
        }
        let component = float_in(range);
        let value = node.value();
        if value.is_array() {
            let elements = read.array(node)?;
            if elements.len() != 3 {
                read.refuse(node, Code::from(&codes::VEC3));
                return None;
            }
            let mut out = [0.0_f32; 3];
            let mut failed = false;
            for (index, slot) in out.iter_mut().enumerate() {
                match elements.element(index).and_then(|n| component(&n, read)) {
                    Some(read) => *slot = read,
                    None => failed = true,
                }
            }
            (!failed).then(|| Vec3::new(out[0], out[1], out[2]))
        } else if value.is_object() {
            let members = read.object(node)?;
            let mut out = [0.0_f32; 3];
            let mut failed = false;
            for (slot, key) in out.iter_mut().zip(["x", "y", "z"]) {
                if let Some(member) = members.get(key).filter(|n| !n.value().is_null()) {
                    match component(&member, read) {
                        Some(read) => *slot = read,
                        None => failed = true,
                    }
                }
            }
            unknown(read, &members, &["x", "y", "z"]);
            (!failed).then(|| Vec3::new(out[0], out[1], out[2]))
        } else {
            read.refuse(node, Code::from(&codes::VEC3));
            None
        }
    }
}
