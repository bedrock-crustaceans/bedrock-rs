//! Writing a texture set, and the chain's view of the newest struct: the payload object and
//! the document with its envelope. A layer the value does not hold is not written.

use facet_value::{VArray, VObject};

use super::set::{LayerKind, LayerValue, Rgb, Rgba, TextureName, TextureSet};
use super::set_read::{PAYLOAD, Schema, V1_21_30, read_layers};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value, WriteStyle, write_json};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned};
use bedrock_addon_support::texture::jsoncpp::read_lenient;

impl Versioned for TextureSet {
    const VERSION: FormatVersion = V1_21_30;
}

impl VersionStruct for TextureSet {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        parse_layers(payload, Schema::V1_21_30, &cx.at, d)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        Ok(write_layers(self))
    }
}

/// The layers of the payload text under `schema`: the schema pass alone, for the chain. The
/// file's own rules (version, envelope, post-validation) are the loader's.
pub(super) fn parse_layers(
    payload: JsonSlice<'_>,
    schema: Schema,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<TextureSet> {
    let read = read_lenient(payload.text.as_bytes()).ok();
    let Some(layers) = read.as_ref().and_then(|read| read.value.as_object()) else {
        let code = DiagnosticCode::InvalidType { text: None };
        let location = Location::new(None, None, at.clone(), None);
        d.push(Diagnostic::new(
            code,
            Severity::Error,
            Phase::Parse,
            location,
        ));
        return None;
    };
    read_layers(layers, schema, at, d)
}

/// A channel as the integer the client stores: rounded and held to 0 to 255.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the value is clamped to 0..=255 first"
)]
fn channel(value: f32) -> u8 {
    if value.is_nan() {
        return 0;
    }
    value.round().clamp(0.0, 255.0) as u8
}

/// A texture name as JSON.
fn name(name: &TextureName) -> Value {
    Value::from(name.0.as_str())
}

/// An array of channels as JSON.
fn array(channels: &[f32]) -> Value {
    let mut out = VArray::new();
    for value in channels {
        out.push(Value::from(i64::from(channel(*value))));
    }
    out.into()
}

/// A four-channel layer as JSON: the name, `#AARRGGBB`, or `[r, g, b, a]`.
fn rgba(value: &LayerValue<Rgba>) -> Value {
    match value {
        LayerValue::Texture(texture) => name(texture),
        LayerValue::Hex(c) => Value::from(
            format!(
                "#{:02X}{:02X}{:02X}{:02X}",
                channel(c.a),
                channel(c.r),
                channel(c.g),
                channel(c.b)
            )
            .as_str(),
        ),
        LayerValue::Array(c) => array(&[c.r, c.g, c.b, c.a]),
    }
}

/// A three-channel layer as JSON: the name, `#RRGGBB`, or `[r, g, b]`.
fn rgb(value: &LayerValue<Rgb>) -> Value {
    match value {
        LayerValue::Texture(texture) => name(texture),
        LayerValue::Hex(c) => Value::from(
            format!(
                "#{:02X}{:02X}{:02X}",
                channel(c.r),
                channel(c.g),
                channel(c.b)
            )
            .as_str(),
        ),
        LayerValue::Array(c) => array(&[c.r, c.g, c.b]),
    }
}

/// The payload object: each layer the set holds, in the schema's order.
pub(super) fn write_layers(set: &TextureSet) -> Value {
    let mut object = VObject::new();
    for layer in LayerKind::ALL {
        let value = match layer {
            LayerKind::Color => set.color.as_ref().map(rgba),
            LayerKind::Normal => set.normal.as_ref().map(name),
            LayerKind::Heightmap => set.heightmap.as_ref().map(name),
            LayerKind::Mer => set.metalness_emissive_roughness.as_ref().map(rgb),
            LayerKind::Mers => set
                .metalness_emissive_roughness_subsurface
                .as_ref()
                .map(rgba),
        };
        if let Some(value) = value {
            object.insert(layer.key(), value);
        }
    }
    object.into()
}

impl TextureSet {
    /// The whole document: `format_version` 1.21.30 and the payload.
    pub(super) fn document(&self) -> Value {
        let mut object = VObject::new();
        object.insert("format_version", V1_21_30.to_string());
        object.insert(PAYLOAD, write_layers(self));
        object.into()
    }

    /// Writes the document into `out`.
    pub(super) fn write_into(
        &self,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<(), EncodeError> {
        write_json(&self.document(), out, style)
    }
}
