//! The scalar duties of the walk: the misread scalars, and the pre-pass's judgment.

use facet::{Def, Shape, Type, UserType};

use super::untagged::{Scalar, payload_shape};
use super::{Cursor, Misreading, ScalarCall, Value};
use crate::json::scalar::{Declared, Refusal, ScalarRules, Scalars, Token, Verdict, judge};

/// How a member of the scalar type `ty` misread the scalar `json`, if it did: the typed
/// parse took the value, so what facet refuses is not asked about.
pub(super) fn misreading(ty: facet::ScalarType, json: &Value) -> Option<Misreading> {
    use facet::ScalarType as T;
    let is_text = matches!(ty, T::String | T::Str | T::CowStr | T::Char);
    let Some(number) = json.as_number() else {
        let same = if json.is_string() {
            is_text
        } else if json.is_bool() {
            matches!(ty, T::Bool)
        } else {
            // `null` is the default of the type; an object or array is facet's to refuse.
            !json.is_null() || matches!(ty, T::Unit)
        };
        return (!same).then_some(Misreading::WrongType);
    };
    let number = number.to_f64_lossy();
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the nearest f32 is the question"
    )]
    match ty {
        T::F32 => (number as f32)
            .is_infinite()
            .then_some(Misreading::BeyondFloat),
        T::F64 => number.is_infinite().then_some(Misreading::BeyondFloat),
        T::Bool => Some(Misreading::WrongType),
        _ => is_text.then_some(Misreading::WrongType),
    }
}

/// The pre-pass's judgment of the scalar `json` for a member of the scalar type `ty`, pushed
/// when it is not read as written.
pub(super) fn judge_scalar(
    ty: facet::ScalarType,
    json: &Value,
    rules: ScalarRules,
    cursor: &mut Cursor,
) {
    if let (Some(array), false) = (json.as_array(), rules.scalars == Scalars::ConvertAndTell) {
        cursor.scalars.push(ScalarCall {
            path: cursor.path.clone(),
            verdict: Verdict::Refused(Refusal::WrongType),
            declared: Declared::of(ty),
            arm: false,
            list: Some(array.len()),
        });
        return;
    }
    let (Some(declared), Some(token)) = (Declared::of(ty), Token::of(json)) else {
        return;
    };
    let verdict = judge(declared, token, rules);
    if verdict != Verdict::Keep {
        cursor.scalars.push(ScalarCall {
            path: cursor.path.clone(),
            verdict,
            declared: Some(declared),
            arm: false,
            list: None,
        });
    }
}

/// The arm of an untagged enum that reads the scalar `token`, as the pre-pass chooses it under
/// `rules`, or `None` when no arm with a declared scalar type reads it (then the enum's own
/// read, or the refusal pushed here, says what it is).
///
/// facet-format (rc.7) reads a scalar into an untagged enum by its variants in order: `null`
/// into the first unit variant, a string into the unit variant of its name, else the first
/// newtype variant around a scalar (or an enum of unit variants) whose type takes the token
/// (`scalar_matches_shape`: a string that parses into the type is taken), else the first such
/// variant whatever the token. Under [`Scalars::ConvertAndTell`] that is the arm. Under the
/// other profiles an arm takes the token when the strict rule does (the `JsonCpp` one under
/// [`Scalars::JsonCpp`]): the first such arm in order, and when none does and every scalar arm
/// has a declared scalar type, the token is refused at the enum (the `StrictShape` check the
/// block family made before the typed read).
pub(super) fn scalar_arm(
    variants: &'static [facet::Variant],
    token: Token<'_>,
    rules: ScalarRules,
    cursor: &mut Cursor,
) -> Option<usize> {
    if matches!(token, Token::Null) && variants.iter().any(|v| v.data.fields.is_empty()) {
        return None;
    }
    if let Token::Str(text) = token
        && variants
            .iter()
            .any(|v| v.data.fields.is_empty() && v.effective_name() == text)
    {
        return None;
    }
    // The newtype variants facet reads a scalar as (`VariantFormat::NewtypeScalar`), with the
    // declared type of each that has one.
    let arms: Vec<(usize, &'static Shape, Option<Declared>)> = variants
        .iter()
        .enumerate()
        .filter_map(|(index, variant)| {
            let payload = see_through_pointer(payload_shape(variant)?);
            let scalar = payload.scalar_type();
            (scalar.is_some() || is_unit_enum(payload))
                .then(|| (index, payload, scalar.and_then(Declared::of)))
        })
        .collect();
    if rules.scalars == Scalars::ConvertAndTell {
        let json = token_value(token);
        let scalar = Scalar::of(&json)?;
        let chosen = arms
            .iter()
            .find(|(_, payload, _)| scalar.match_quality(payload).is_some())
            .or_else(|| arms.first())?;
        return chosen.2.map(|_| chosen.0);
    }
    let arm_rules = match rules.scalars {
        Scalars::JsonCpp => rules,
        _ => rules.with_scalars(Scalars::Strict),
    };
    let takes =
        |declared: Declared| !matches!(judge(declared, token, arm_rules), Verdict::Refused(_));
    if let Some((index, _, _)) = arms
        .iter()
        .find(|(_, _, declared)| declared.is_some_and(takes))
    {
        return Some(*index);
    }
    if !arms.is_empty() && arms.iter().all(|(_, _, declared)| declared.is_some()) {
        cursor.scalars.push(ScalarCall {
            path: cursor.path.clone(),
            verdict: Verdict::Refused(Refusal::WrongType),
            declared: None,
            arm: true,
            list: None,
        });
    }
    None
}

/// The value of a scalar token, for the scalar matching facet's untagged read does.
fn token_value(token: Token<'_>) -> Value {
    match token {
        Token::Null => Value::NULL,
        Token::Bool(b) => Value::from(b),
        Token::UInt(u) => Value::from(u),
        Token::Int(i) => Value::from(i),
        Token::Real(x) => Value::from(x),
        Token::Str(text) => Value::from(text),
    }
}

/// `shape` without pointers (facet-solver `deref_pointer`).
fn see_through_pointer(mut shape: &'static Shape) -> &'static Shape {
    while let Def::Pointer(def) = shape.def {
        match def.pointee() {
            Some(pointee) => shape = pointee,
            None => break,
        }
    }
    shape
}

/// An enum of unit variants only, which facet reads from a scalar (facet-solver
/// `is_unit_enum_shape`).
fn is_unit_enum(shape: &'static Shape) -> bool {
    match shape.ty {
        Type::User(UserType::Enum(en)) => en.variants.iter().all(|v| v.data.fields.is_empty()),
        _ => false,
    }
}
