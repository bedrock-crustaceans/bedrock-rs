//! The scalar duties of the walk: the misread scalars, and the pre-pass's judgment.

use super::{Cursor, Misreading, ScalarCall, Value};
use crate::json::arms::{ScalarArm, choose_scalar_arm};
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
/// other profiles an arm takes the token when the strict rule does: the first such arm in order,
/// and when none does and every scalar arm has a declared scalar type, the token is refused at
/// the enum (the `StrictShape` check the block family made before the typed read).
pub(super) fn scalar_arm(
    variants: &'static [facet::Variant],
    token: Token<'_>,
    rules: ScalarRules,
    cursor: &mut Cursor,
) -> Option<usize> {
    match choose_scalar_arm(variants, token, rules) {
        ScalarArm::Arm {
            index,
            declared: true,
        } => Some(index),
        ScalarArm::Refused => {
            cursor.scalars.push(ScalarCall {
                path: cursor.path.clone(),
                verdict: Verdict::Refused(Refusal::WrongType),
                declared: None,
                arm: true,
                list: None,
            });
            None
        }
        ScalarArm::Arm { .. } | ScalarArm::Unit | ScalarArm::None => None,
    }
}
