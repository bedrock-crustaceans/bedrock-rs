//! Components that hold one value and also read the bare value: `{"value": x}`
//! or `x`.

use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc, keep_members, parse_at};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::versioning::ConvertMode;
use facet::Facet;

use super::{ItemComponent, typed};
use bedrock_addon_core::diagnostic::{
    Diagnostic, Diagnostics, JsonPath, Location, Phase, Severity, code_for_deserialize_error_kind,
};
use bedrock_addon_core::document::Extra;

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

/// Says what the bare `value` read from `slice` is misread as (a string where a number
/// belongs, a number beyond a float), as a member of the object form is told.
pub(crate) fn tell_misread<S: for<'a> Facet<'a>>(
    value: &mut S,
    slice: JsonSlice<'_>,
    d: &mut Diagnostics,
) {
    if let Ok(json) = from_str_jsonc::<Value>(slice.text) {
        for misread in keep_members(value, &json).1 {
            d.push(misread.finding(&JsonPath::new()));
        }
    }
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
    match parse_at::<T::Scalar>(slice) {
        Ok(mut value) => {
            tell_misread(&mut value, slice, d);
            Some(T::from_scalar(value).into())
        }
        Err(error) => {
            let blank = slice.text.trim().is_empty();
            // A scalar of another type is a type error that refuses the item; the one
            // context without `strict_components` is the write-back probe of `write.rs`,
            // which reads only the `Unrecognized*` codes.
            d.push(Diagnostic::new(
                code_for_deserialize_error_kind(&error.error.kind, false, blank),
                Severity::Error,
                Phase::Parse,
                Location::new(None, None, JsonPath::new(), Some(error.span)),
            ));
            None
        }
    }
}

/// Declares a one-value component.
macro_rules! value_component {
    (
        $(#[$meta:meta])*
        $name:ident($ty:ty) $(= $default:literal)?;
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Default, PartialEq, ::facet::Facet)]
        pub struct $name {
            /// The value (JSON `value`, or the bare value).
            #[facet(default $(= $default)?)]
            pub value: $ty,
            /// Unknown members.
            #[facet(skip, rename = "\u{0}extra")]
            pub extra: ::bedrock_addon_core::document::Extra,
        }

        impl $name {
            /// The component holding `value`.
            pub fn new(value: $ty) -> Self {
                Self { value, extra: ::bedrock_addon_core::document::Extra::default() }
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
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct FireResistant {
    /// Whether it resists (JSON `value`).
    #[facet(default)]
    pub value: bool,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:max_stack_size`: how many items fit in one slot, 1–64.
///
/// JSON `{"value": n}` or `n`. An object without `value` reads as 1: vanilla's writer leaves
/// out a default, and writes a stack size of 1 as `{}`.
// (inferred) `max_stack_size`
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MaxStackSize {
    /// The stack size (JSON `value`).
    #[facet(default = 1)]
    pub value: i32,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl MaxStackSize {
    /// A stack size of `value`.
    #[must_use]
    pub fn new(value: i32) -> Self {
        Self {
            value,
            extra: Extra::default(),
        }
    }
}

impl Default for MaxStackSize {
    fn default() -> Self {
        Self::new(1)
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
