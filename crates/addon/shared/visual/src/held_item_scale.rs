//! `description.held_item_scale`: a number, or Molang from engine 1.26.30.

use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::{JsonType, ScalarShape, StrictError};
use bedrock_addon_core::molang::{Molang, MolangFields, MolangSlot, MolangSource, MolangVisitor};
use bedrock_addon_core::schema::FromJson;

/// `held_item_scale`: how much larger or smaller items held by the entity are drawn; 1.0 when
/// absent.
///
/// A number is the scale; vanilla stores it only when it is above 0. From engine 1.26.30 any
/// other value (a string, or a boolean) is a Molang expression compiled with the pack's version,
/// and a result at or below 0 draws at 1.0. `null`, an array or an object is not a scale.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(proxy = ScalarShape)]
#[repr(u8)]
pub enum HeldItemScale {
    /// A JSON number, integer or real. It is written back as a real (`2` as `2.0`), the
    /// shortest that reads back to the same `f32` (`2.678`): the variant keeps no token.
    Number(f32),
    /// A string or a boolean: Molang (engine 1.26.30 and up).
    Molang(Molang),
}

impl TryFrom<ScalarShape> for HeldItemScale {
    type Error = StrictError;

    fn try_from(json: ScalarShape) -> Result<Self, StrictError> {
        // A number takes the "> 0" path; any other non-null value is read as a Molang
        // expression at the pack's version.
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)] // the nearest f32
        let number = match json {
            ScalarShape::UInt(u) => u as f32,
            ScalarShape::Int(i) => i as f32,
            ScalarShape::Number(x) => x as f32,
            ScalarShape::String(text) => {
                return Ok(Self::Molang(Molang::Expr(
                    MolangSource::string_without_context(text),
                )));
            }
            ScalarShape::Bool(b) => return Ok(Self::Molang(Molang::Bool(b))),
            ScalarShape::Null => {
                return Err(StrictError::WrongType {
                    expected: "number or Molang",
                    found: JsonType::Null,
                });
            }
        };
        if number.is_finite() {
            Ok(Self::Number(number))
        } else {
            Err(StrictError::OutOfRange)
        }
    }
}

impl TryFrom<&HeldItemScale> for ScalarShape {
    type Error = StrictError;

    fn try_from(scale: &HeldItemScale) -> Result<Self, StrictError> {
        // The double whose shortest decimal is the `f32`'s (2.678, not the widened
        // 2.677999973297119): it reads back to the same `f32` (as for a
        // strict float).
        let number = |x: f32| {
            if x.is_finite() {
                let shortest = format!("{x:?}")
                    .parse::<f64>()
                    .unwrap_or_else(|_| f64::from(x));
                Ok(ScalarShape::Number(shortest))
            } else {
                Err(StrictError::NonFinite)
            }
        };
        match scale {
            // The token kind is not kept, so an integer token is written as a real.
            HeldItemScale::Number(x) | HeldItemScale::Molang(Molang::Const(x)) => number(*x),
            HeldItemScale::Molang(Molang::Bool(b)) => Ok(ScalarShape::Bool(*b)),
            // The member has no object form; a source built with one is written as its text.
            HeldItemScale::Molang(Molang::Expr(source)) => {
                Ok(ScalarShape::String(source.as_str().to_owned()))
            }
            // `MolangValueRepr` is non-exhaustive; a future variant has no member form.
            HeldItemScale::Molang(_) => Err(StrictError::WrongType {
                expected: "number or Molang",
                found: JsonType::Null,
            }),
        }
    }
}

/// A number as the `f32` it is (one beyond it the largest finite float of its sign, told), a
/// string or a boolean as Molang; `null`, an array and an object are a wrong type.
impl FromJson for HeldItemScale {
    #[allow(clippy::cast_possible_truncation)] // the nearest f32 is the reading
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if read.stopped() {
            return None;
        }
        let value = node.value();
        if let Some(number) = value.as_number() {
            let x = number.to_f64_lossy() as f32;
            if x.is_finite() {
                return Some(Self::Number(x));
            }
            let beyond =
                bedrock_addon_core::molang::beyond_f32_warning(node.path().clone(), node.span());
            read.push(beyond);
            return Some(Self::Number(f32::MAX.copysign(x)));
        }
        if let Some(text) = value.as_string() {
            let source = MolangSource::string_without_context(text.as_str());
            return Some(Self::Molang(Molang::Expr(source)));
        }
        if let Some(b) = value.as_bool() {
            return Some(Self::Molang(Molang::Bool(b)));
        }
        read.wrong_type(node);
        None
    }
}

bedrock_addon_core::impl_schema_leaf!(HeldItemScale);

/// The Molang form is the member's one value, under the spec the walk's field names.
impl MolangFields for HeldItemScale {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        if let Self::Molang(scale) = self {
            let spec = *visitor.default_spec();
            scale.visit_slot(&spec, visitor);
        }
    }
}
