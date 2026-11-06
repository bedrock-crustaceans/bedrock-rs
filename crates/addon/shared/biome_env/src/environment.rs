//! `environment_attributes` of the beta document (`minecraft:biome` and
//! `minecraft:client_biome` alike): experimental attributes, each a value and how it combines
//! with the layers below it.

use facet::Facet;

use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_shared_shapes::HexOr;

/// The environment attributes of a beta biome or client biome: the four experimental keys
/// below, each optional.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct EnvironmentAttributes {
    /// JSON `minecraft:experimental_bool_attr`.
    #[facet(rename = "minecraft:experimental_bool_attr")]
    pub experimental_bool_attr: Option<BoolAttribute>,
    /// JSON `minecraft:experimental_color_attr`.
    #[facet(rename = "minecraft:experimental_color_attr")]
    pub experimental_color_attr: Option<ColorAttribute>,
    /// JSON `minecraft:experimental_float_attr_1`.
    #[facet(rename = "minecraft:experimental_float_attr_1")]
    pub experimental_float_attr_1: Option<FloatAttribute>,
    /// JSON `minecraft:experimental_float_attr_2`.
    #[facet(rename = "minecraft:experimental_float_attr_2")]
    pub experimental_float_attr_2: Option<FloatAttribute>,
}

/// A boolean attribute; both members are required.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct BoolAttribute {
    /// JSON `value`.
    pub value: bool,
    /// JSON `operation`.
    pub operation: BoolOperation,
}

/// A colour attribute; both members are required.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct ColorAttribute {
    /// JSON `value`: `"#RRGGBBAA"` or `[r, g, b, a]`.
    pub value: HexOr<[i32; 4]>,
    /// JSON `operation`.
    pub operation: ColorOperation,
}

/// A number attribute; both members are required.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct FloatAttribute {
    /// JSON `operation`.
    pub operation: FloatOperation,
    /// JSON `value`.
    pub value: f32,
}

/// How a boolean attribute combines with the one below it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
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

impl BoolAttribute {
    /// `value`, combined by `operation`.
    pub fn new(value: bool, operation: BoolOperation) -> Self {
        Self { value, operation }
    }
}

impl ColorAttribute {
    /// `value`, combined by `operation`.
    pub fn new(value: HexOr<[i32; 4]>, operation: ColorOperation) -> Self {
        Self { value, operation }
    }
}

impl FloatAttribute {
    /// `value`, combined by `operation`.
    pub fn new(value: f32, operation: FloatOperation) -> Self {
        Self { operation, value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_attribute_constructors_read_back() {
        let attributes = EnvironmentAttributes {
            experimental_bool_attr: Some(BoolAttribute::new(true, BoolOperation::Xor)),
            experimental_color_attr: Some(ColorAttribute::new(
                HexOr::Array([1, 2, 3, 4]),
                ColorOperation::Multiply,
            )),
            experimental_float_attr_1: Some(FloatAttribute::new(0.5, FloatOperation::Maximum)),
            ..Default::default()
        };
        let text = bedrock_addon_core::json::to_json(&attributes).expect("writes");
        let read: EnvironmentAttributes = bedrock_addon_core::json::from_str(&text).expect("reads");
        assert_eq!(read, attributes);
    }
}
