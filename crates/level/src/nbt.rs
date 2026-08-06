//! Decoding NBT into this crate's types, with the accommodations Bedrock's own
//! save data needs.
//!
//! Everything here exists for one reason: **an NBT `Byte` tag used as a flag is
//! false only when it is zero.** Any other value — `2`, `-1`, `0xFF` — is true.
//! That is the rule Bedrock's own save data is written under, and it is the rule
//! this crate has always decoded by.
//!
//! nbtx reads a `bool` field strictly instead: only the byte `1` is `true`, and
//! *every* other byte, `2` included, is `false`. Letting the ~120 `bool` fields
//! in `LevelSettings`, `PlayerData` and the block entities take that reading
//! would flip `0x02..=0xFF` from true to false everywhere, silently and with no
//! compiler error to catch it.
//!
//! There is no per-field hook to reach for: the derive's attributes are
//! declarative, so a field cannot name a conversion function, and a wrapper type
//! does not help either — nbtx resolves scalars by `TypeId`, so a newtype over
//! `i8` reflects as a struct and encodes as a compound rather than a `Byte`.
//!
//! So the rule is reapplied here instead, once, generically: decode into a
//! dynamic [`nbtx::Value`], walk it against the target type's shape, and
//! canonicalise every `Byte` that is destined for a `bool` field to `1` before
//! handing the tree over. The public field types stay `bool` and no call site
//! changes. The same walk reconciles numeric tag widths — see [`retag_integer`]
//! and [`retag_float`] — since the game does not write one fixed tag per key.

use std::io::Read;

use facet::{Def, Facet, ScalarType, Shape, Type, UserType};
use nbtx::Value;

/// Reads a `T` from little-endian (on-disk) NBT, applying [`normalize_scalars`]
/// on the way.
///
/// A drop-in replacement for [`nbtx::from_le_bytes`] for any type with a `bool`
/// or a narrow integer field. It costs one intermediate [`Value`] tree; use
/// nbtx's function directly for a type that has neither.
pub fn from_le_bytes<'f, T: Facet<'f>>(reader: &mut impl Read) -> Result<T, nbtx::Error> {
    let value: Value = nbtx::from_le_bytes(reader)?;
    from_value(value)
}

/// Builds a `T` from an already-decoded [`Value`], applying
/// [`normalize_scalars`] on the way.
///
/// Use this when the tree has already been read — to inspect a discriminating
/// key before choosing the target type, say — so it is not decoded twice.
pub fn from_value<'f, T: Facet<'f>>(mut value: Value) -> Result<T, nbtx::Error> {
    normalize_scalars(&mut value, T::SHAPE);
    nbtx::from_value(value)
}

/// Reconciles `value`'s scalars with the targets in `shape` they are bound for:
/// a `Byte` headed for a `bool` is canonicalised to `1` when it is non-zero; an
/// integer, whether it fills a number or selects an enum variant, is re-tagged
/// to the width its target expects when it fits (see [`retag_integer`]); and a
/// float is re-tagged to its field's width (see [`retag_float`]). Every other
/// node is left untouched.
///
/// The walk mirrors the one nbtx's own decoder performs — `Option` unwraps to its
/// inner shape, a compound matches keys against `effective_name()` so
/// `#[facet(rename)]`/`rename_all` are honoured, a list recurses into its element
/// shape — and simply stops wherever the shape says nothing about the value (a
/// dynamic [`Value`] field, an unknown key, a tag/shape mismatch that nbtx will
/// report for itself). It is therefore purely a normalisation pass: it never
/// adds, drops or reorders anything, and it cannot turn a decodable document
/// into an undecodable one.
///
/// `i8`/`u8` fields are deliberately untouched — that is the whole reason this
/// walks the shape rather than clamping every `Byte` in the tree.
pub fn normalize_scalars(value: &mut Value, shape: &'static Shape) {
    // An `Option<T>` field carries `T`'s shape once the key is present.
    if let Def::Option(def) = shape.def {
        normalize_scalars(value, def.t());
        return;
    }

    if let Some(scalar) = ScalarType::try_from_shape(shape) {
        match scalar {
            // Bedrock's rule: zero is false, anything else is true. nbtx reads
            // only `1` as true, so fold every other non-zero byte onto it.
            ScalarType::Bool => {
                if let Value::Byte(byte) = value
                    && *byte != 0
                {
                    *byte = 1;
                }
            }
            ScalarType::I8 => retag_integer(value, IntWidth::Byte { signed: true }),
            ScalarType::I16 => retag_integer(value, IntWidth::Short { signed: true }),
            ScalarType::I32 => retag_integer(value, IntWidth::Int { signed: true }),
            ScalarType::I64 => retag_integer(value, IntWidth::Long { signed: true }),
            ScalarType::F32 | ScalarType::F64 => retag_float(value, scalar),
            _ => {}
        }
        return;
    }

    // An enum arrives as the tag its `#[facet(nbtx::variant_as(...))]` declares.
    // A numeric mode is a fixed-width integer holding the discriminant, so it
    // needs the same reconciling a plain integer field does.
    if let Type::User(UserType::Enum(_)) = shape.ty {
        if let Some(width) = variant_as_width(shape) {
            retag_integer(value, width);
        }
        return;
    }

    match value {
        Value::Compound(entries) => {
            if let Type::User(UserType::Struct(st)) = shape.ty {
                for (key, entry) in entries.iter_mut() {
                    // Keys with no matching field are left alone; nbtx decides
                    // whether they are an error or are skipped.
                    if let Some(field) = st
                        .fields
                        .iter()
                        .find(|f| f.effective_name().as_bytes() == key.as_slice())
                    {
                        normalize_scalars(entry, field.shape());
                    }
                }
            } else if let Def::Map(def) = shape.def {
                for (_, entry) in entries.iter_mut() {
                    normalize_scalars(entry, def.v());
                }
            }
        }
        // Only a `List` can hold `bool` elements: nbtx tags a sequence by its
        // element type, and `Vec<u8>`/`Vec<i32>`/`Vec<i64>` — the three typed
        // arrays — are not `bool`.
        Value::List(items) => {
            let elem = match shape.def {
                Def::List(def) => Some(def.t()),
                Def::Array(def) => Some(def.t()),
                Def::Slice(def) => Some(def.t()),
                _ => None,
            };
            if let Some(elem) = elem {
                for item in items {
                    normalize_scalars(item, elem);
                }
            }
        }
        _ => {}
    }
}

/// The integer tag a field is read from, and whether the number it carries is
/// signed.
///
/// An unsigned target stores its bit pattern in the signed tag — discriminant
/// 200 under a `u8` mode is `Byte(-56)` — so the range a value has to fit
/// depends on the sign as well as the width.
#[derive(Clone, Copy)]
enum IntWidth {
    Byte { signed: bool },
    Short { signed: bool },
    Int { signed: bool },
    Long { signed: bool },
}

/// The width an enum's `#[facet(nbtx::variant_as(<mode>))]` declares, or `None`
/// for the `str` mode and for an enum that declares nothing (which nbtx rejects
/// on its own).
fn variant_as_width(shape: &'static Shape) -> Option<IntWidth> {
    let mode = shape
        .attributes
        .iter()
        .find(|attr| attr.ns == Some("nbtx") && attr.key == "variant_as")?
        .get_as::<Shape>()?;

    let id = mode.id;
    Some(if id == <i8 as Facet>::SHAPE.id {
        IntWidth::Byte { signed: true }
    } else if id == <u8 as Facet>::SHAPE.id {
        IntWidth::Byte { signed: false }
    } else if id == <i16 as Facet>::SHAPE.id {
        IntWidth::Short { signed: true }
    } else if id == <u16 as Facet>::SHAPE.id {
        IntWidth::Short { signed: false }
    } else if id == <i32 as Facet>::SHAPE.id {
        IntWidth::Int { signed: true }
    } else if id == <u32 as Facet>::SHAPE.id {
        IntWidth::Int { signed: false }
    } else if id == <i64 as Facet>::SHAPE.id {
        IntWidth::Long { signed: true }
    } else if id == <u64 as Facet>::SHAPE.id {
        IntWidth::Long { signed: false }
    } else {
        // `str` mode: the variant arrives as its name, not a number.
        return None;
    })
}

/// Re-tags an integer value to the width its target expects, when the value
/// fits.
///
/// Bedrock does not write one fixed tag per key: a chest's `Findable` is a
/// `Byte` in a world the game wrote, against an `i32` field, and other numeric
/// keys vary the same way — including the ones that select an enum variant, such
/// as a structure block's `data`. nbtx pairs each tag with exactly one width and
/// rejects the rest, so the two are reconciled here, for every numeric target,
/// since a tag whose value fits is not ambiguous anywhere.
///
/// A value that does not fit is left alone, and nbtx then reports the mismatch,
/// so nothing is silently truncated. This is a *read* accommodation only:
/// writing always emits the tag the Rust type implies, so a `Byte` read into an
/// `i32` is written back as an `Int`.
fn retag_integer(value: &mut Value, width: IntWidth) {
    let widened = match *value {
        Value::Byte(v) => i64::from(v),
        Value::Short(v) => i64::from(v),
        Value::Int(v) => i64::from(v),
        Value::Long(v) => v,
        _ => return,
    };

    // An unsigned target keeps the bit pattern rather than the number, matching
    // how nbtx narrows a discriminant on the way out.
    let retagged = match width {
        IntWidth::Byte { signed: true } => i8::try_from(widened).ok().map(Value::Byte),
        IntWidth::Byte { signed: false } => u8::try_from(widened)
            .ok()
            .map(|v| Value::Byte(v.cast_signed())),
        IntWidth::Short { signed: true } => i16::try_from(widened).ok().map(Value::Short),
        IntWidth::Short { signed: false } => u16::try_from(widened)
            .ok()
            .map(|v| Value::Short(v.cast_signed())),
        IntWidth::Int { signed: true } => i32::try_from(widened).ok().map(Value::Int),
        IntWidth::Int { signed: false } => u32::try_from(widened)
            .ok()
            .map(|v| Value::Int(v.cast_signed())),
        IntWidth::Long { signed: true } => Some(Value::Long(widened)),
        IntWidth::Long { signed: false } => u64::try_from(widened)
            .ok()
            .map(|v| Value::Long(v.cast_signed())),
    };

    if let Some(retagged) = retagged {
        *value = retagged;
    }
}

/// Re-tags a `Float` as a `Double` or the reverse, to match the width of the
/// field it is bound for.
///
/// The same tag inconsistency [`retag_integer`] handles, in the float pair: an
/// item frame's `ItemRotation` is an `f32` field that a record may carry as
/// either tag. Unlike the integers this needs no range check — a float
/// conversion in either direction is defined for every value, saturating to an
/// infinity rather than trapping — so the cast is unconditional and, when
/// narrowing, lossy in the low bits.
fn retag_float(value: &mut Value, scalar: ScalarType) {
    match (scalar, &*value) {
        (ScalarType::F32, Value::Double(v)) => *value = Value::Float(*v as f32),
        (ScalarType::F64, Value::Float(v)) => *value = Value::Double(f64::from(*v)),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nbtx::Compound;

    #[derive(Facet, Debug, PartialEq)]
    #[facet(rename_all = "PascalCase")]
    struct Inner {
        #[facet(rename = "wasPickedUp")]
        was_picked_up: bool,
        count: i8,
    }

    #[derive(Facet, Debug, PartialEq)]
    struct Outer {
        flag: bool,
        raw: i8,
        maybe: Option<bool>,
        items: Vec<Inner>,
        opaque: nbtx::Value,
    }

    fn outer(flag_byte: i8) -> Value {
        Value::Compound(Compound::from_iter([
            ("flag".into(), Value::Byte(flag_byte)),
            ("raw".into(), Value::Byte(flag_byte)),
            ("maybe".into(), Value::Byte(flag_byte)),
            (
                "items".into(),
                Value::List(vec![Value::Compound(Compound::from_iter([
                    ("wasPickedUp".into(), Value::Byte(flag_byte)),
                    ("Count".into(), Value::Byte(flag_byte)),
                ]))]),
            ),
            ("opaque".into(), Value::Byte(flag_byte)),
        ]))
    }

    /// The byte `2` is what nbtx would read as `false`; every `bool` in the tree
    /// must still come back `true`, at every nesting level.
    #[test]
    fn non_zero_byte_is_true_at_every_depth() {
        let decoded: Outer = from_value(outer(2)).unwrap();
        assert_eq!(
            decoded,
            Outer {
                flag: true,
                raw: 2,
                maybe: Some(true),
                items: vec![Inner {
                    was_picked_up: true,
                    count: 2
                }],
                opaque: Value::Byte(2),
            }
        );
    }

    /// Zero is the one byte that means false.
    #[test]
    fn zero_byte_is_false() {
        let decoded: Outer = from_value(outer(0)).unwrap();
        assert!(!decoded.flag);
        assert_eq!(decoded.maybe, Some(false));
        assert!(!decoded.items[0].was_picked_up);
    }

    /// A `Byte` that is not headed for a `bool` keeps its exact value, including
    /// inside a dynamic `Value` field the shape says nothing about.
    #[test]
    fn integer_bytes_are_left_alone() {
        let mut value = outer(-7);
        normalize_scalars(&mut value, Outer::SHAPE);
        let Value::Compound(entries) = &value else {
            panic!("expected a compound");
        };
        assert_eq!(entries[bstr::BStr::new("raw")], Value::Byte(-7));
        assert_eq!(entries[bstr::BStr::new("opaque")], Value::Byte(-7));
        assert_eq!(entries[bstr::BStr::new("flag")], Value::Byte(1));
    }

    #[derive(Facet, Debug, PartialEq)]
    struct Widths {
        small: i8,
        big: i32,
        huge: i64,
    }

    /// A number arrives under whichever tag the game felt like writing; it fills
    /// the field as long as it fits.
    #[test]
    fn integer_tags_are_reconciled_with_the_field_width() {
        let value = Value::Compound(Compound::from_iter([
            ("small".into(), Value::Int(7)),
            ("big".into(), Value::Byte(-3)),
            ("huge".into(), Value::Short(1000)),
        ]));
        assert_eq!(
            from_value::<Widths>(value).unwrap(),
            Widths {
                small: 7,
                big: -3,
                huge: 1000
            }
        );
    }

    /// A number too wide for its field is left as it is, so nbtx reports the
    /// mismatch rather than this pass truncating it.
    #[test]
    fn out_of_range_integers_are_not_truncated() {
        let value = Value::Compound(Compound::from_iter([
            ("small".into(), Value::Int(300)),
            ("big".into(), Value::Byte(0)),
            ("huge".into(), Value::Byte(0)),
        ]));
        assert!(from_value::<Widths>(value).is_err());
    }

    #[derive(Facet, Debug, PartialEq)]
    #[facet(nbtx::variant_as(i32))]
    #[repr(i32)]
    enum Mode {
        Data = 0,
        Save = 1,
    }

    #[derive(Facet, Debug, PartialEq)]
    #[facet(nbtx::variant_as(str))]
    #[repr(u8)]
    enum Named {
        Base,
    }

    #[derive(Facet, Debug, PartialEq)]
    struct Tagged {
        mode: Mode,
        named: Named,
        rotation: f32,
        precise: f64,
    }

    /// The number selecting an enum variant arrives under whichever tag the game
    /// wrote, exactly like a plain number, and a float likewise.
    #[test]
    fn enum_and_float_tags_are_reconciled_too() {
        let value = Value::Compound(Compound::from_iter([
            // `Int` is the width `variant_as(i32)` expects; a `Byte` still fills it.
            ("mode".into(), Value::Byte(1)),
            ("named".into(), Value::String("Base".into())),
            ("rotation".into(), Value::Double(1.5)),
            ("precise".into(), Value::Float(0.5)),
        ]));
        assert_eq!(
            from_value::<Tagged>(value).unwrap(),
            Tagged {
                mode: Mode::Save,
                named: Named::Base,
                rotation: 1.5,
                precise: 0.5,
            }
        );
    }

    /// A discriminant no variant claims stays as it is, so nbtx reports it
    /// rather than this pass inventing a variant.
    #[test]
    fn unknown_discriminants_still_fail() {
        let value = Value::Compound(Compound::from_iter([
            ("mode".into(), Value::Byte(9)),
            ("named".into(), Value::String("Base".into())),
            ("rotation".into(), Value::Float(0.0)),
            ("precise".into(), Value::Double(0.0)),
        ]));
        assert!(from_value::<Tagged>(value).is_err());
    }
}
