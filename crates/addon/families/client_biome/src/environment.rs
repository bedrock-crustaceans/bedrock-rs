//! `minecraft:client_biome.environment_attributes` of the beta document: experimental
//! attributes, each a value and how it combines with the layers below it.

use facet::Facet;

use super::color::Color255Rgba;
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{StrictBool, StrictFloat};

/// The environment attributes of a beta client biome: exactly the four experimental keys
/// below, each optional.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct EnvironmentAttributes {
    /// JSON `minecraft:experimental_bool_attr`.
    #[facet(default, rename = "minecraft:experimental_bool_attr")]
    pub experimental_bool_attr: Option<BoolAttribute>,
    /// JSON `minecraft:experimental_color_attr`.
    #[facet(default, rename = "minecraft:experimental_color_attr")]
    pub experimental_color_attr: Option<ColorAttribute>,
    /// JSON `minecraft:experimental_float_attr_1`.
    #[facet(default, rename = "minecraft:experimental_float_attr_1")]
    pub experimental_float_attr_1: Option<FloatAttribute>,
    /// JSON `minecraft:experimental_float_attr_2`.
    #[facet(default, rename = "minecraft:experimental_float_attr_2")]
    pub experimental_float_attr_2: Option<FloatAttribute>,
    /// Unknown attributes.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// A boolean attribute; both members are required.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct BoolAttribute {
    /// JSON `value`.
    pub value: StrictBool,
    /// JSON `operation`.
    pub operation: BoolOperation,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// A colour attribute; both members are required.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct ColorAttribute {
    /// JSON `value`: `"#RRGGBBAA"` or `[r, g, b, a]`.
    pub value: Color255Rgba,
    /// JSON `operation`.
    pub operation: ColorOperation,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// A number attribute; both members are required.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct FloatAttribute {
    /// JSON `operation`.
    pub operation: FloatOperation,
    /// JSON `value`.
    pub value: StrictFloat<f32>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// How a boolean attribute combines with the one below it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
#[allow(missing_docs, reason = "each variant is its JSON spelling")]
pub enum BoolOperation {
    Override,
    AlphaBlend,
    And,
    Nand,
    Or,
    Nor,
    Xor,
    Xnor,
}

/// How a colour attribute combines with the one below it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
#[allow(missing_docs, reason = "each variant is its JSON spelling")]
pub enum ColorOperation {
    Override,
    AlphaBlend,
    Add,
    Subtract,
    Multiply,
}

/// How a number attribute combines with the one below it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
#[allow(missing_docs, reason = "each variant is its JSON spelling")]
pub enum FloatOperation {
    Override,
    AlphaBlend,
    Add,
    Subtract,
    Multiply,
    Minimum,
    Maximum,
}
