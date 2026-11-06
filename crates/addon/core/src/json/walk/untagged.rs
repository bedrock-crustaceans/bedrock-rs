//! The arm of an untagged enum facet reads a JSON value as, without the typed value.

use std::borrow::Cow;
use std::ptr;

use facet::{Def, Field, Shape};

use super::{Lookup, Value, find_field, struct_fields};

/// The arm of an untagged enum facet reads a JSON value as, mirroring facet-format rc.7's
/// untagged deserialiser without the typed value:
/// - an object: the arms that read an object (struct variants, and newtype variants around a
///   named struct or a map) are the candidates; each key, in document order, that some
///   remaining candidate has narrows the candidates to those having it, and the first time one
///   candidate is left it is the arm (facet-solver `see_key`); a key no remaining candidate has
///   is ignored. When the remaining candidates give the key fields of different types and its
///   value is a scalar, they are narrowed further to those whose field type takes the value best
///   (`scalar_narrowing`); when none takes it, facet stops and reads the first object arm. When
///   no key singles one out, facet falls back to the first object arm;
/// - an array: the first list arm;
/// - a scalar: no members to walk.
///
/// Residual uncertainty: a map arm is taken to have no keys, flattened fields and
/// `alias` are matched as `find_field` matches them, and a newtype around an `Option` or a
/// pointer is not an object arm, as facet classifies it; these follow facet's source and are not
/// confirmed against every input.
pub(super) fn untagged_arm(variants: &'static [facet::Variant], json: &Value) -> Option<usize> {
    if json.is_array() {
        return variants.iter().position(|v| {
            payload_shape(v)
                .is_some_and(|s| matches!(see_through(s).def, Def::List(_) | Def::Array(_)))
        });
    }
    let object = json.as_object()?;
    let arms: Vec<(usize, ObjectArm)> = variants
        .iter()
        .enumerate()
        .filter_map(|(index, variant)| match object_fields(variant) {
            ObjectArm::None => None,
            arm => Some((index, arm)),
        })
        .collect();
    let first = arms.first()?.0;
    let mut candidates: Vec<&(usize, ObjectArm)> = arms.iter().collect();
    for (key, member) in object {
        let with_key: Vec<&(usize, ObjectArm)> = candidates
            .iter()
            .copied()
            .filter(|(_, arm)| match arm {
                ObjectArm::Fields(fields) => {
                    !matches!(find_field(fields, key.as_str()), Lookup::Unknown)
                }
                ObjectArm::Map | ObjectArm::None => false,
            })
            .collect();
        if with_key.is_empty() {
            continue;
        }
        candidates = with_key;
        if let [(index, _)] = candidates.as_slice() {
            return Some(*index);
        }
        if let Some(taking) = scalar_narrowing(&candidates, key.as_str(), member) {
            candidates.retain(|(_, arm)| {
                key_shape(arm, key.as_str()).is_some_and(|s| taking.iter().any(|t| ptr::eq(*t, s)))
            });
            match candidates.as_slice() {
                [] => return Some(first), // facet's `SatisfyResult::NoMatch`
                [(index, _)] => return Some(*index),
                _ => {}
            }
        }
    }
    Some(first)
}

/// facet-solver `KeyResult::Ambiguous` and facet-format rc.7 `select_best_ambiguous_scalar_shapes`:
/// when the candidates' fields for `key` have more than one type (by shape) and the member is a
/// scalar, the field types that take it best: the best match quality (0 exact, 1 a string parsed
/// into a number or bool), then the lowest specificity score (`u8` before `u32` before `u64`).
/// `None` when the key is not ambiguous or the member is an object or array (facet then keeps
/// every candidate). An empty list means no candidate takes the value.
fn scalar_narrowing(
    candidates: &[&(usize, ObjectArm)],
    key: &str,
    member: &Value,
) -> Option<Vec<&'static Shape>> {
    let mut shapes: Vec<&'static Shape> = Vec::new();
    for (_, arm) in candidates {
        if let Some(shape) = key_shape(arm, key)
            && !shapes.iter().any(|s| ptr::eq(*s, shape))
        {
            shapes.push(shape);
        }
    }
    if shapes.len() < 2 {
        return None;
    }
    let scalar = Scalar::of(member)?;
    let rated: Vec<(&'static Shape, u8, u64)> = shapes
        .into_iter()
        .filter_map(|shape| {
            scalar
                .match_quality(shape)
                .map(|quality| (shape, quality, specificity_score(shape)))
        })
        .collect();
    let Some(quality) = rated.iter().map(|(_, q, _)| *q).min() else {
        return Some(Vec::new());
    };
    let score = rated
        .iter()
        .filter(|(_, q, _)| *q == quality)
        .map(|(_, _, s)| *s)
        .min();
    Some(
        rated
            .into_iter()
            .filter(|(_, q, s)| *q == quality && Some(*s) == score)
            .map(|(shape, _, _)| shape)
            .collect(),
    )
}

/// The type of the field an object arm reads `key` into (a flattened struct's field included).
fn key_shape(arm: &ObjectArm, key: &str) -> Option<&'static Shape> {
    fn in_fields(fields: &'static [Field], key: &str) -> Option<&'static Shape> {
        match find_field(fields, key) {
            Lookup::Field(_, field) => Some(field.shape()),
            Lookup::Flattened(..) => fields
                .iter()
                .filter(|f| f.is_flattened() && !f.should_skip_deserializing())
                .find_map(|f| in_fields(struct_fields(f.shape())?, key)),
            Lookup::Unknown => None,
        }
    }
    match arm {
        ObjectArm::Fields(fields) => in_fields(fields, key),
        ObjectArm::Map | ObjectArm::None => None,
    }
}

/// A JSON scalar as facet-json's parser hands it to the solver.
pub(super) enum Scalar<'v> {
    Null,
    Bool,
    /// A non-negative integer token.
    U64(u64),
    /// A negative integer token.
    I64,
    /// A real token, or an integer token past 64 bits: the door reads that as the double it
    /// spells (`json/read.rs`).
    F64,
    /// A string.
    Str(Cow<'v, str>),
}

impl<'v> Scalar<'v> {
    /// The scalar `value` is, or `None` for an object or array.
    pub(super) fn of(value: &'v Value) -> Option<Self> {
        if value.is_null() {
            return Some(Self::Null);
        }
        if value.as_bool().is_some() {
            return Some(Self::Bool);
        }
        if let Some(text) = value.as_string() {
            return Some(Self::Str(Cow::Borrowed(text.as_str())));
        }
        let number = value.as_number()?;
        Some(if number.is_float() {
            Self::F64
        } else if let Some(u) = number.to_u64() {
            Self::U64(u)
        } else if number.to_i64().is_some() {
            Self::I64
        } else {
            // An integer past 64 bits: no text read through the door gives one.
            Self::F64
        })
    }

    /// facet-format rc.7 `scalar_match_quality`: 0 for a direct match, 1 for a string that
    /// parses into the type, `None` when the type does not take the value. Only primitive scalar
    /// types take a scalar (an `Option` takes only `null`; a struct or proxied type none).
    pub(super) fn match_quality(&self, shape: &'static Shape) -> Option<u8> {
        use facet::ScalarType as T;
        let Some(ty) = shape.scalar_type() else {
            return (matches!(self, Self::Null) && matches!(shape.def, Def::Option(_)))
                .then_some(0);
        };
        let takes = match self {
            Self::Null => matches!(ty, T::Unit),
            Self::Bool => matches!(ty, T::Bool),
            Self::F64 => matches!(ty, T::F32 | T::F64),
            // An unsigned token: any unsigned type, a signed one it fits.
            Self::U64(u) => match ty {
                T::U8 | T::U16 | T::U32 | T::U64 | T::U128 | T::USize => true,
                T::I8 => i8::try_from(*u).is_ok(),
                T::I16 => i16::try_from(*u).is_ok(),
                T::I32 => i32::try_from(*u).is_ok(),
                T::I64 | T::I128 | T::ISize => i64::try_from(*u).is_ok(),
                _ => false,
            },
            // A negative token: any signed type (facet checks no range), no unsigned one.
            Self::I64 => matches!(ty, T::I8 | T::I16 | T::I32 | T::I64 | T::I128 | T::ISize),
            Self::Str(text) => {
                if matches!(ty, T::String | T::Str | T::CowStr | T::Char) {
                    return Some(0);
                }
                return parses(ty, text).then_some(1);
            }
        };
        takes.then_some(0)
    }
}

/// Whether the type's `FromStr` (facet's parse function for these scalars) takes `text`.
fn parses(ty: facet::ScalarType, text: &str) -> bool {
    use facet::ScalarType as T;
    match ty {
        T::Bool => text.parse::<bool>().is_ok(),
        T::F32 => text.parse::<f32>().is_ok(),
        T::F64 => text.parse::<f64>().is_ok(),
        T::U8 => text.parse::<u8>().is_ok(),
        T::U16 => text.parse::<u16>().is_ok(),
        T::U32 => text.parse::<u32>().is_ok(),
        T::U64 => text.parse::<u64>().is_ok(),
        T::U128 => text.parse::<u128>().is_ok(),
        T::USize => text.parse::<usize>().is_ok(),
        T::I8 => text.parse::<i8>().is_ok(),
        T::I16 => text.parse::<i16>().is_ok(),
        T::I32 => text.parse::<i32>().is_ok(),
        T::I64 => text.parse::<i64>().is_ok(),
        T::I128 => text.parse::<i128>().is_ok(),
        T::ISize => text.parse::<isize>().is_ok(),
        // `()`, `ConstTypeId` and the network types: no field of a pack document has them.
        _ => false,
    }
}

/// facet-solver `specificity_score`: lower is a narrower type.
fn specificity_score(shape: &'static Shape) -> u64 {
    match shape.type_identifier {
        "u8" | "i8" => 8,
        "u16" | "i16" => 16,
        "u32" | "i32" | "f32" => 32,
        "u64" | "i64" | "f64" | "usize" | "isize" => 64,
        "u128" | "i128" => 128,
        _ => 1000,
    }
}

/// What an arm reads a JSON object as.
enum ObjectArm {
    /// A struct (a struct variant, or a newtype variant around a named struct).
    Fields(&'static [Field]),
    /// A map.
    Map,
    /// Not an object.
    None,
}

/// How facet classifies a variant for object input (facet-solver `VariantFormat::from_variant`:
/// `Struct`, or `NewtypeStruct` for a newtype around a named struct or a map).
fn object_fields(variant: &'static facet::Variant) -> ObjectArm {
    if variant.data.kind == facet::StructKind::Struct {
        return ObjectArm::Fields(variant.data.fields);
    }
    let Some(shape) = payload_shape(variant) else {
        return ObjectArm::None;
    };
    if shape.effective_proxy(None).is_some() {
        return ObjectArm::None;
    }
    if matches!(shape.def, Def::Map(_)) {
        return ObjectArm::Map;
    }
    struct_fields(shape).map_or(ObjectArm::None, ObjectArm::Fields)
}

/// The payload shape of a newtype variant.
pub(super) fn payload_shape(variant: &'static facet::Variant) -> Option<&'static Shape> {
    match variant.data.fields {
        [field] if variant.data.kind != facet::StructKind::Struct => Some(field.shape()),
        _ => None,
    }
}

/// `shape` without `Option` and pointers.
fn see_through(mut shape: &'static Shape) -> &'static Shape {
    loop {
        shape = match shape.def {
            Def::Option(option) => option.t(),
            Def::Pointer(def) => match def.pointee() {
                Some(pointee) => pointee,
                None => return shape,
            },
            _ => return shape,
        };
    }
}
