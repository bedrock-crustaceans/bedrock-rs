//! Writing a render controller file back: only the members the value
//! holds, each Molang value in the form it was read. `materials` and `part_visibility` are
//! written one pattern per element, which keeps the order the game applies them in.

use facet_value::{VArray, VObject};
use indexmap::IndexMap;

use super::{BonePattern, RenderController, RenderControllerArrays, RenderControllerFile, Rgba};
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_core::molang::{Molang, MolangSource};

fn molang(value: &Molang) -> Result<Value, EncodeError> {
    bedrock_addon_core::molang::write(value).map_err(|what| EncodeError::NotRepresentable {
        at: JsonPath::new(),
        what,
    })
}

fn source(source: &MolangSource) -> Value {
    Value::from(source.as_str())
}

/// A `materials` value. The empty text is what the game reads a JSON number as, and a written
/// `""` would be compiled by the schema and fail the whole file, so it is written as a number.
fn material(text: &MolangSource) -> Value {
    if text.is_empty() {
        Value::from(0_i64)
    } else {
        source(text)
    }
}

fn sources(list: &[MolangSource]) -> Value {
    list.iter().map(source).collect::<VArray>().into()
}

fn bones<T>(
    list: &[(BonePattern, T)],
    write: impl Fn(&T) -> Result<Value, EncodeError>,
) -> Result<Value, EncodeError> {
    let mut array = VArray::new();
    for (pattern, value) in list {
        let mut element = VObject::new();
        element.insert(pattern.0.as_str(), write(value)?);
        array.push(Value::from(element));
    }
    Ok(array.into())
}

fn array_kind(map: &IndexMap<String, Vec<MolangSource>>) -> Value {
    let mut object = VObject::new();
    for (name, list) in map {
        object.insert(name.as_str(), sources(list));
    }
    object.into()
}

fn arrays(arrays: &RenderControllerArrays) -> Value {
    let mut object = VObject::new();
    for (key, map) in [
        ("geometries", &arrays.geometries),
        ("materials", &arrays.materials),
        ("textures", &arrays.textures),
    ] {
        // A kind without an array fails the file, so an empty map is an absent kind.
        if !map.is_empty() {
            object.insert(key, array_kind(map));
        }
    }
    object.into()
}

fn color(color: &Rgba) -> Result<Value, EncodeError> {
    let mut object = VObject::new();
    for (key, channel) in [
        ("r", &color.r),
        ("g", &color.g),
        ("b", &color.b),
        ("a", &color.a),
    ] {
        if let Some(channel) = channel {
            object.insert(key, molang(channel)?);
        }
    }
    Ok(object.into())
}

fn pair(values: &[Molang; 2]) -> Result<Value, EncodeError> {
    values
        .iter()
        .map(molang)
        .collect::<Result<VArray, _>>()
        .map(Value::from)
}

impl RenderController {
    /// The controller as JSON.
    pub(crate) fn to_value(&self) -> Result<Value, EncodeError> {
        let mut object = VObject::new();
        if let Some(value) = &self.arrays {
            object.insert("arrays", arrays(value));
        }
        object.insert("geometry", source(&self.geometry));
        if !self.part_visibility.is_empty() {
            object.insert("part_visibility", bones(&self.part_visibility, molang)?);
        }
        if !self.materials.is_empty() {
            object.insert("materials", bones(&self.materials, |s| Ok(material(s)))?);
        }
        // An empty `textures` is not written: the game rejects `[]` with the whole file, and
        // without the member it drops only this controller.
        if !self.textures.is_empty() {
            object.insert("textures", sources(&self.textures));
        }
        for (key, value) in [
            ("color", &self.color),
            ("overlay_color", &self.overlay_color),
            ("on_fire_color", &self.on_fire_color),
            ("is_hurt_color", &self.is_hurt_color),
        ] {
            if let Some(value) = value {
                object.insert(key, color(value)?);
            }
        }
        if let Some(uv_anim) = &self.uv_anim {
            let mut uv = VObject::new();
            uv.insert("offset", pair(&uv_anim.offset)?);
            uv.insert("scale", pair(&uv_anim.scale)?);
            object.insert("uv_anim", Value::from(uv));
        }
        if let Some(value) = &self.light_color_multiplier {
            object.insert("light_color_multiplier", molang(value)?);
        }
        if let Some(value) = self.ignore_lighting {
            object.insert("ignore_lighting", value);
        }
        if self.filter_lighting.is_some() {
            object.insert("filter_lighting", true);
        }
        if let Some(value) = self.rebuild_animation_matrices {
            object.insert("rebuild_animation_matrices", value);
        }
        Ok(object.into())
    }
}

impl RenderControllerFile {
    /// The whole file as JSON.
    pub(crate) fn to_value(&self) -> Result<Value, EncodeError> {
        // Loaded and dropped controllers together, in the game's member order (sorted keys):
        // the order a reader gives them again.
        let mut all: Vec<(&String, &RenderController)> = self
            .render_controllers
            .iter()
            .chain(&self.dropped)
            .collect();
        all.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        if all.is_empty() {
            // The schema requires a controller: `"render_controllers": {}` fails the file.
            let mut at = JsonPath::new();
            at.push("render_controllers");
            return Err(EncodeError::NotRepresentable {
                at,
                what: bedrock_addon_core::json::Unrepresentable::Unsupported,
            });
        }
        // One key in both maps (a file built by hand): the object has the key once, so one of
        // the two controllers would be lost.
        if let Some(twice) = all.windows(2).find(|pair| pair[0].0 == pair[1].0) {
            let mut at = JsonPath::new();
            at.push("render_controllers");
            at.push(twice[0].0.as_str());
            return Err(EncodeError::NotRepresentable {
                at,
                what: bedrock_addon_core::json::Unrepresentable::Unsupported,
            });
        }
        let mut controllers = VObject::new();
        for (key, controller) in all {
            controllers.insert(key.as_str(), controller.to_value()?);
        }
        let mut object = VObject::new();
        object.insert("format_version", self.format_version.to_string());
        object.insert("render_controllers", Value::from(controllers));
        Ok(object.into())
    }
}
