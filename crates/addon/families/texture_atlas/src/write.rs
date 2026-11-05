//! Reading one atlas file on its own and writing an atlas back: `Content` and `Placed` for
//! [`TextureAtlas`]. A member the value does not hold is not written.

use facet_value::{VArray, VObject};

use super::read::read_atlas;
use super::types::{
    AtlasItem, AtlasKind, AtlasTextures, Color, Isotropic, TextureAtlas, TextureElement,
    TextureLayer, Tint, TintMapColor, VariationEntry,
};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics};
use bedrock_addon_core::document::{
    Content, Document, Extra, JsonContent, Placed, Rejected, unversioned,
};
use bedrock_addon_core::json::{EncodeError, Value, WriteStyle, write_json};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use bedrock_addon_support::texture::jsoncpp::read_lenient;

impl Content for TextureAtlas {
    const NAME: &'static str = "texture atlas";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one atlas file on its own, as a stack of one pack would: `item_texture.json` is
    /// the item atlas, any other name the terrain atlas. Rejected when the file is no JSON or
    /// names no `texture_name`.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        // A number no double holds is read as the largest finite one, with a warning.
        bedrock_addon_core::document::FiniteText::load(bytes, |bytes| {
            let origin = unversioned::lone(path);
            let kind = if path.as_str().ends_with("item_texture.json") {
                AtlasKind::Item
            } else {
                AtlasKind::Terrain
            };
            let mut diagnostics = Diagnostics::default();
            let read = match read_lenient(bytes) {
                Ok(read) => read,
                Err(message) => {
                    let code = DiagnosticCode::MalformedInput {
                        message: message.to_string(),
                        text: None,
                    };
                    diagnostics.push(Diagnostic::error(code).in_file(&origin));
                    return Err(unversioned::rejected(origin, diagnostics));
                }
            };
            match read_atlas(kind, &read.value, &mut diagnostics) {
                Some(atlas) => Ok(unversioned::document(atlas, origin, diagnostics)),
                None => Err(unversioned::rejected(origin, diagnostics)),
            }
        })
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        write_json(&atlas_json(self), out, style)
    }
}

impl JsonContent for TextureAtlas {}

impl Placed for TextureAtlas {
    /// `textures/terrain_texture.json` or `textures/item_texture.json`, by the kind.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(self.kind.path())
    }
}

/// Appends the members of `extra`.
fn put_extra(object: &mut VObject, extra: &Extra) {
    for (key, value) in extra.iter() {
        object.insert(key, value.clone());
    }
}

/// Inserts `value` when there is one.
fn put(object: &mut VObject, key: &str, value: Option<impl Into<Value>>) {
    if let Some(value) = value {
        object.insert(key, value);
    }
}

/// The atlas as JSON: vanilla's own member order, then the unread members.
fn atlas_json(atlas: &TextureAtlas) -> Value {
    let mut root = VObject::new();
    put(
        &mut root,
        "resource_pack_name",
        atlas.ignored.get("resource_pack_name").cloned(),
    );
    root.insert("texture_name", atlas.texture_name.as_str());
    put(&mut root, "padding", atlas.padding.map(i64::from));
    put(
        &mut root,
        "num_mip_levels",
        atlas.num_mip_levels.map(i64::from),
    );
    put(
        &mut root,
        "default_leather_color",
        atlas.default_leather_color.as_deref(),
    );
    put(
        &mut root,
        "default_horse_leather_color",
        atlas.default_horse_leather_color.as_deref(),
    );
    // Not core's `write_envelope`: `format_version` is a member the game ignores, written back
    // as read among the others, and there is no payload key.
    put(
        &mut root,
        "format_version",
        atlas.ignored.get("format_version").cloned(),
    );
    if let Some(data) = &atlas.texture_data {
        let mut tiles = VObject::new();
        for (key, item) in data {
            tiles.insert(key.as_str(), item_json(item));
        }
        root.insert("texture_data", tiles.into_value());
    }
    put_extra(&mut root, &atlas.extra);
    root.into_value()
}

/// One tile.
fn item_json(item: &AtlasItem) -> Value {
    let mut node = VObject::new();
    put(&mut node, "quad", item.quad);
    put(&mut node, "additive", item.additive);
    put(&mut node, "mip_fade_amount", item.mip_fade_amount);
    put(
        &mut node,
        "mip_fade_color",
        item.mip_fade_color.map(color_json),
    );
    let textures = match &item.textures {
        AtlasTextures::One(element) => element_json(element),
        AtlasTextures::Many(elements) => {
            let mut array = VArray::new();
            for element in elements {
                array.push(element_json(element));
            }
            array.into_value()
        }
    };
    node.insert("textures", textures);
    put_extra(&mut node, &item.extra);
    node.into_value()
}

/// One element.
fn element_json(element: &TextureElement) -> Value {
    match element {
        TextureElement::Path(path) => Value::from(path.as_str()),
        TextureElement::Layer(layer) => layer_json(layer),
        TextureElement::Invalid(value) => value.clone(),
        TextureElement::Variations {
            variations,
            ignored,
        } => {
            let mut array = VArray::new();
            for entry in variations {
                array.push(match entry {
                    VariationEntry::Path(path) => Value::from(path.as_str()),
                    VariationEntry::Layer(layer) => layer_json(layer),
                    VariationEntry::Invalid(value) => value.clone(),
                });
            }
            let mut object = VObject::new();
            object.insert("variations", array.into_value());
            put_extra(&mut object, ignored);
            object.into_value()
        }
    }
}

/// One object layer.
fn layer_json(layer: &TextureLayer) -> Value {
    let mut object = VObject::new();
    put(&mut object, "path", layer.path.as_deref());
    put(&mut object, "tint_map_path", layer.tint_map_path.as_deref());
    put(&mut object, "overlay_color", layer.overlay_color.as_deref());
    put(
        &mut object,
        "tint_color",
        layer.tint_color.as_ref().map(|tint| match tint {
            Tint::Color(color) => color_json(*color),
            Tint::Map(map) => tint_map_json(map),
        }),
    );
    put(
        &mut object,
        "tint_base_color",
        layer.tint_base_color.as_ref().map(tint_map_json),
    );
    put(&mut object, "mip_fade_amount", layer.mip_fade_amount);
    put(
        &mut object,
        "mip_fade_color",
        layer.mip_fade_color.map(color_json),
    );
    put(&mut object, "weight", layer.weight);
    put(
        &mut object,
        "isotropic",
        layer.isotropic.as_ref().map(|isotropic| match isotropic {
            Isotropic::All(all) => Value::from(*all),
            Isotropic::Faces(faces) => {
                let mut map = VObject::new();
                for (face, on) in faces {
                    map.insert(face.name(), *on);
                }
                map.into_value()
            }
        }),
    );
    put_extra(&mut object, &layer.extra);
    object.into_value()
}

/// A tint map.
fn tint_map_json(map: &TintMapColor) -> Value {
    let mut object = VObject::new();
    put(&mut object, "r_color", map.r_color.map(color_json));
    put(&mut object, "g_color", map.g_color.map(color_json));
    put(&mut object, "b_color", map.b_color.map(color_json));
    put(&mut object, "a_color", map.a_color.map(color_json));
    object.into_value()
}

/// A colour: an array of bytes, or `#` and the eight hex digits vanilla reads back to the same
/// bits.
fn color_json(color: Color) -> Value {
    let array = |bytes: &[u8]| {
        let mut array = VArray::new();
        for byte in bytes {
            array.push(i64::from(*byte));
        }
        array.into_value()
    };
    match color {
        Color::Rgb(bytes) => array(&bytes),
        Color::Rgba(bytes) => array(&bytes),
        Color::Hex(bits) => Value::from(format!("#{bits:08x}")),
    }
}
