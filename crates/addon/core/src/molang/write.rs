//! Writing a Molang field back as an untyped `Value`: exactly the form
//! that was read.

use super::{JsonScalar, Molang, MolangList};
use crate::json::{Unrepresentable, Value};
use facet_value::{VArray, VObject};

/// The JSON value of `value` in the form it was read (`MolangValueRepr::write`): a `Const` as
/// the number an author writes for it ([`ConstToken`]: `1`, `0.1`), a `Bool` as a bool, a
/// string form as a string, an object form as `{"expression", "version"}`. A `Const` that is
/// not finite has no JSON form.
///
/// Vanilla's own writer folds constant expressions and writes every other one as the object
/// form; a writer that must match it converts first.
// hidden: Molang pass machinery of the crate family, outside SemVer
#[doc(hidden)]
pub fn write(value: &Molang) -> Result<Value, Unrepresentable> {
    scalar_value(value.write())
}

/// The JSON value of a value-or-array field: one value, or an array of the elements' values.
// hidden: Molang pass machinery of the crate family, outside SemVer
#[doc(hidden)]
pub fn write_list(list: &MolangList) -> Result<Value, Unrepresentable> {
    match list {
        MolangList::One(value) => write(value),
        MolangList::Many(elements) => elements
            .iter()
            .map(write_list)
            .collect::<Result<VArray, _>>()
            .map(Value::from),
    }
}

/// How a finite Molang constant is written: the token an author writes for the value, as far
/// as the constant (an `f32`) holds it.
///
/// A constant does not keep its spelling, so the writer picks the one that leaves the most
/// documents as they were written: an integral value is an integer (`1`, not `1.0`), any other
/// the shortest decimal that reads back as the same `f32` (`0.1`, not the widened
/// `0.10000000149011612`). `1.0` is therefore rewritten `1` and `0.10` rewritten `0.1`; the
/// value never changes.
// Vanilla reads a Molang number as a number whatever its token: the int / real tag of the
// token is not looked at, unlike a plain float member's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ConstToken {
    /// An integer token: an integral value an `f32` holds exactly as an integer (below 2^24
    /// in magnitude), except negative zero.
    Integer(i32),
    /// A real token: the shortest decimal of the `f32`, with `.0` when it is integral.
    Real(f32),
}

impl ConstToken {
    /// The first integer magnitude an `f32` does not hold next to all its neighbours.
    const EXACT_INTEGERS: f32 = 16_777_216.0;

    /// The token of the finite constant `x`.
    pub(crate) fn of(x: f32) -> Self {
        let integral = x.fract() == 0.0 && x.abs() < Self::EXACT_INTEGERS;
        let negative_zero = x == 0.0 && x.is_sign_negative();
        if integral && !negative_zero {
            #[allow(
                clippy::cast_possible_truncation,
                reason = "integral and below 2^24 in magnitude"
            )]
            Self::Integer(x as i32)
        } else {
            Self::Real(x)
        }
    }

    /// The token as a `Value`. A real is the `f64` of its shortest decimal when that narrows
    /// back to the same `f32` (a `Value` holds an `f64`), else the `f32` widened exactly.
    fn value(self) -> Value {
        match self {
            Self::Integer(n) => Value::from(i64::from(n)),
            Self::Real(x) => {
                let widened = f64::from(x);
                let shortest = format!("{x:?}").parse::<f64>().unwrap_or(widened);
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "narrowing is the check: the text must read back as `x`"
                )]
                let reads_back = (shortest as f32).to_bits() == x.to_bits();
                Value::from(if reads_back { shortest } else { widened })
            }
        }
    }
}

fn scalar_value(scalar: JsonScalar) -> Result<Value, Unrepresentable> {
    Ok(match scalar {
        JsonScalar::Number(x) if x.is_nan() => return Err(Unrepresentable::NaN),
        JsonScalar::Number(x) if x == f64::INFINITY => return Err(Unrepresentable::Infinity),
        JsonScalar::Number(x) if x == f64::NEG_INFINITY => {
            return Err(Unrepresentable::NegativeInfinity);
        }
        // `write` gives a `Const` widened exactly, so narrowing gives the constant back.
        #[allow(
            clippy::cast_possible_truncation,
            reason = "the number is an `f32` widened"
        )]
        JsonScalar::Number(x) => ConstToken::of(x as f32).value(),
        JsonScalar::Bool(b) => Value::from(b),
        JsonScalar::String(text) => Value::from(&*text),
        JsonScalar::Object {
            expression,
            version,
        } => {
            let mut object = VObject::new();
            object.insert("expression", Value::from(&*expression));
            object.insert("version", Value::from(i64::from(version.0)));
            Value::from(object)
        }
        // `write` never gives another shape; a newer molang's would have no JSON form here.
        _ => return Err(Unrepresentable::Unsupported),
    })
}
