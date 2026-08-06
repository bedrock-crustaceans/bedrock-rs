//! Decoding NBT into this crate's types, with Bedrock's boolean rule applied.
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
//! changes. The same walk reconciles integer widths — see [`retag_integer`].

use std::io::Read;

use facet::{Def, Facet, ScalarType, Shape, Type, UserType};
use nbtx::Value;

/// Reads a `T` from little-endian (on-disk) NBT, treating any non-zero `Byte`
/// destined for a `bool` field as `true`.
///
/// A drop-in replacement for [`nbtx::from_le_bytes`] for types that contain
/// `bool` fields. It costs one intermediate [`Value`] tree; use nbtx's function
/// directly for types with no `bool` anywhere in them.
pub fn from_le_bytes<'f, T: Facet<'f>>(reader: &mut impl Read) -> Result<T, nbtx::Error> {
    let value: Value = nbtx::from_le_bytes(reader)?;
    from_value(value)
}

/// Builds a `T` from an already-decoded [`Value`], treating any non-zero `Byte`
/// destined for a `bool` field as `true`.
///
/// Use this when the tree has already been read (to inspect a discriminating key
/// before choosing the target type, say) so it is not decoded twice.
pub fn from_value<'f, T: Facet<'f>>(mut value: Value) -> Result<T, nbtx::Error> {
    canonicalize_flags(&mut value, T::SHAPE);
    nbtx::from_value(value)
}

/// Rewrites every `Byte` in `value` that a field of `shape` would read as a
/// `bool` to the canonical `1`, leaving every other node untouched.
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
pub fn canonicalize_flags(value: &mut Value, shape: &'static Shape) {
    // An `Option<T>` field carries `T`'s shape once the key is present.
    if let Def::Option(def) = shape.def {
        canonicalize_flags(value, def.t());
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
            ScalarType::I8 | ScalarType::I16 | ScalarType::I32 | ScalarType::I64 => {
                retag_integer(value, scalar);
            }
            _ => {}
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
                        canonicalize_flags(entry, field.shape());
                    }
                }
            } else if let Def::Map(def) = shape.def {
                for (_, entry) in entries.iter_mut() {
                    canonicalize_flags(entry, def.v());
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
                    canonicalize_flags(item, elem);
                }
            }
        }
        _ => {}
    }
}

/// Re-tags an integer value to the width its target field expects, when the
/// value fits.
///
/// Bedrock does not write one fixed tag per field: a chest's `Findable` is a
/// `Byte` in a world the game wrote, against an `i32` field, and other numeric
/// keys vary the same way. nbtx pairs each tag with exactly one Rust width and
/// rejects the rest, so the two are reconciled here — for every field, since a
/// tag whose value fits the field is not ambiguous anywhere.
///
/// A value that does not fit is left alone, and nbtx then reports the mismatch,
/// so nothing is silently truncated. This is a *read* accommodation only:
/// writing always emits the tag the Rust type implies, so a `Byte` read into an
/// `i32` is written back as an `Int`.
fn retag_integer(value: &mut Value, scalar: ScalarType) {
    let widened = match *value {
        Value::Byte(v) => i64::from(v),
        Value::Short(v) => i64::from(v),
        Value::Int(v) => i64::from(v),
        Value::Long(v) => v,
        _ => return,
    };

    let retagged = match scalar {
        ScalarType::I8 => i8::try_from(widened).ok().map(Value::Byte),
        ScalarType::I16 => i16::try_from(widened).ok().map(Value::Short),
        ScalarType::I32 => i32::try_from(widened).ok().map(Value::Int),
        ScalarType::I64 => Some(Value::Long(widened)),
        _ => None,
    };

    if let Some(retagged) = retagged {
        *value = retagged;
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
        canonicalize_flags(&mut value, Outer::SHAPE);
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
}
