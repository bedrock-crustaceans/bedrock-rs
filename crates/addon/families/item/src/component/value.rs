//! Components that hold one value and also read the bare value: `{"value": x}`
//! or `x`.

use bedrock_addon_core::json::{JsonSlice, ScalarFailure, parse_with_rules};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::versioning::ConvertMode;
use facet::Facet;

use super::{ItemComponent, typed};
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath};
use bedrock_addon_core::schema::{self, Schema};

/// A component that also reads its bare value.
pub(crate) trait ValueComponent: for<'a> Facet<'a> {
    /// The bare value's type.
    type Scalar: for<'a> Facet<'a>;
    /// The component holding `value`.
    fn from_scalar(value: Self::Scalar) -> Self;
}

/// Whether the JSON text (after whitespace and comments) is an object.
pub(crate) fn is_object(text: &str) -> bool {
    let mut rest = text;
    loop {
        rest = rest.trim_start();
        if let Some(after) = rest.strip_prefix("//") {
            rest = after.split_once('\n').map_or("", |(_, tail)| tail);
        } else if let Some(after) = rest.strip_prefix("/*") {
            rest = after.split_once("*/").map_or("", |(_, tail)| tail);
        } else {
            return rest.starts_with('{');
        }
    }
}

/// Reads the bare `value` of a one-value component from `slice` under the slice's scalar rules
/// (`LoadContext::scalar_rules`), and says what it is read as where that is another value than
/// the text writes (a string where a number belongs, a number beyond a float), as a member of
/// the object form is told. A bare value no arm of its type takes is told at the value here.
pub(crate) fn read_bare<S: for<'a> Facet<'a>>(
    slice: JsonSlice<'_>,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Result<S, ScalarFailure> {
    let read = parse_with_rules::<S>(slice, None, cx.scalar_rules).inspect_err(|failure| {
        if let ScalarFailure::Refused(misread) = failure {
            d.push(misread.finding(&JsonPath::new()));
        }
    })?;
    for told in read.told {
        d.push(told.finding(&JsonPath::new()));
    }
    Ok(read.value)
}

/// Parses a [`ValueComponent`] from its object form or its bare value. A bare value of the
/// wrong type drops the component with one warning, as a failed object form does.
pub(crate) fn parse_value<T: ValueComponent + Into<ItemComponent>>(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<ItemComponent> {
    if is_object(slice.text) {
        return typed::<T>(slice, mode, cx, d);
    }
    match read_bare::<T::Scalar>(slice, cx, d) {
        Ok(value) => Some(T::from_scalar(value).into()),
        Err(ScalarFailure::Refused(_)) => None,
        Err(ScalarFailure::Read(error)) => {
            // A scalar of another type is a type error that refuses the item; the one
            // context without `strict_components` is the write-back probe of `write.rs`,
            // which reads only the `Unrecognized*` codes.
            d.push(Diagnostic::error(error.code_with_path(false)).span(error.span));
            None
        }
    }
}

/// Declares a one-value component: its value, its default (`= x`) and the bounds of the value
/// (`[schema::min = x]`), and its bare form.
macro_rules! value_component {
    (
        $(#[$meta:meta])*
        $name:ident($ty:ty) $(= $default:literal)? $([$($bound:tt)+])?;
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq, ::facet::Facet, ::bedrock_addon_core::schema::Schema)]
        #[facet(schema::unknown = "keep")]
        pub struct $name {
            /// The value (JSON `value`, or the bare value).
            #[facet(default $(= $default)? $(, $($bound)+)?)]
            pub value: $ty,
        }

        impl $name {
            /// The component holding `value`.
            pub fn new(value: $ty) -> Self {
                Self { value }
            }
        }

        impl $crate::component::ValueComponent for $name {
            type Scalar = $ty;
            fn from_scalar(value: $ty) -> Self {
                Self::new(value)
            }
        }
    };
}

pub(crate) use value_component;

value_component! {
    /// `minecraft:allow_off_hand`: the item can be held in the off hand.
    AllowOffHand(bool);
}
value_component! {
    /// `minecraft:can_destroy_in_creative`: the item can break blocks in creative mode.
    CanDestroyInCreative(bool);
}
value_component! {
    /// `minecraft:hand_equipped`: the item is rendered like a tool when held.
    HandEquipped(bool);
}
value_component! {
    /// `minecraft:liquid_clipped`: the item interacts with liquid blocks.
    LiquidClipped(bool);
}
value_component! {
    /// `minecraft:should_despawn`: the dropped item despawns.
    ShouldDespawn(bool);
}
value_component! {
    /// `minecraft:stacked_by_data`: stacks of the item are split by aux value.
    StackedByData(bool);
}

/// `minecraft:fire_resistant`: the dropped item is immune to fire and lava (JSON `{"value":
/// bool}` only; no bare form is bound).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct FireResistant {
    /// Whether it resists (JSON `value`).
    #[facet(default)]
    pub value: bool,
}

/// `minecraft:max_stack_size`: how many items fit in one slot, 1–64.
///
/// JSON `{"value": n}` or `n`. An object without `value` reads as 1: vanilla's writer leaves
/// out a default, and writes a stack size of 1 as `{}`.
// (inferred) `max_stack_size`
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MaxStackSize {
    /// The stack size (JSON `value`).
    #[facet(default = 1, schema::min = 1, schema::max = 64)]
    pub value: i32,
}

impl MaxStackSize {
    /// A stack size of `value`.
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

impl ValueComponent for MaxStackSize {
    type Scalar = i32;
    fn from_scalar(value: i32) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::is_object;

    #[test]
    fn an_object_is_recognised_after_comments_and_whitespace() {
        assert!(is_object(" {\"value\": 1}"));
        assert!(is_object("// note\n {}"));
        assert!(is_object("/* x */{}"));
        assert!(!is_object(" 16"));
        assert!(!is_object("\"text\""));
    }
}
