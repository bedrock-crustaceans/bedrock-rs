//! Components that hold one value and also read the bare value: `{"value": x}`
//! or `x`. A one-value component's row says `shorthand = scalar`, and the door reads its bare
//! value as the member (`versioning::read_component`); [`ValueComponent`] is left for the
//! components whose bare value is not their member as written.

use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::versioning::{ConvertMode, read_slice};
use facet::Facet;

use super::{ItemComponent, typed};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::schema::{self, FromJson, Schema};

/// A component whose bare value is not its one member as written, read by hand: `icon`, whose
/// key is its `textures.default`, and the legacy chain's `use_duration` and `icon`, which no
/// row of [`ItemComponent`] reads.
pub(crate) trait ValueComponent: FromJson + for<'a> Facet<'a> {
    /// The bare value's type.
    type Scalar: FromJson;
    /// The component holding `value`.
    fn from_scalar(value: Self::Scalar) -> Self;
}

/// Reads a [`ValueComponent`] from its object form or its bare value (`minecraft:icon`'s
/// row). A bare value of the wrong type drops the component with one error, a type error
/// that refuses the item; a bare value its type refuses is told at the value and dropped.
pub(crate) fn parse_value<T: ValueComponent + Into<ItemComponent>>(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<ItemComponent> {
    if slice.opens_an_object() {
        return typed::<T>(slice, mode, cx, d);
    }
    read_slice::<T::Scalar>(slice, cx, d).map(|value| T::from_scalar(value).into())
}

/// Declares a one-value component: its value, its default (`= x`) and the bounds of the value
/// (`[schema::min = x]`). Its row's `shorthand = scalar` reads the bare form.
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
