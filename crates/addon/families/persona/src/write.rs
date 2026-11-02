//! Writing the family's JSON files back as they were read: the members the model holds, in
//! the game's order, then the members kept as written. A member the author did not write is
//! never written; of a member held both ways (kept as written at load, then set in the model)
//! the model's stands.

use facet_value::{VArray, VObject};

use super::names::{Named, Written};
use super::read::ENVELOPE;
use super::{
    AnimationSource, FaceFlag, GeometrySource, HexColor, PieceMetadata, PieceStoreInfo,
    TextureSource, TintColor, TintColorOrString, Zones,
};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::Value;

/// One object being written.
struct Members(VObject);

impl Members {
    fn new() -> Self {
        Self(VObject::new())
    }

    /// The member `key`, when the model holds it.
    fn put<T>(&mut self, key: &str, value: Option<&T>, write: impl FnOnce(&T) -> Value) {
        if let Some(value) = value {
            self.0.insert(key, write(value));
        }
    }

    /// The members kept as written, but those the model holds by now.
    fn finish(mut self, extra: &Extra) -> Value {
        for (key, value) in extra.iter() {
            if !self.0.contains_key(key) {
                self.0.insert(key, value.clone());
            }
        }
        self.0.into()
    }
}

#[allow(clippy::ptr_arg, reason = "the writer of an `Option<String>` member")]
fn text(value: &String) -> Value {
    Value::from(value.as_str())
}

#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "the writer of an `Option<bool>` member"
)]
fn flag(value: &bool) -> Value {
    Value::from(*value)
}

#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "the writer of an `Option<i32>` member"
)]
fn int(value: &i32) -> Value {
    Value::from(i64::from(*value))
}

fn name<T: Named>(value: &Written<T>) -> Value {
    Value::from(value.as_str())
}

fn color(value: &HexColor) -> Value {
    Value::from(value.as_str())
}

fn list<T>(values: &[T], write: impl Fn(&T) -> Value) -> Value {
    let mut array = VArray::with_capacity(values.len());
    for value in values {
        array.push(write(value));
    }
    array.into()
}

fn zones(value: &Zones) -> Value {
    match value {
        Zones::One(zone) => text(zone),
        Zones::Many(zones) => list(zones, text),
    }
}

fn face(value: &FaceFlag) -> Value {
    match value {
        FaceFlag::Bool(flag) => Value::from(*flag),
        FaceFlag::Text(text) => Value::from(text.as_str()),
    }
}

fn tint(value: &TintColorOrString) -> Value {
    match value {
        TintColorOrString::Channels(channels) => channels.to_value(),
        TintColorOrString::String(one) => color(one),
    }
}

impl TintColor {
    fn to_value(&self) -> Value {
        let mut members = Members::new();
        members.put("r_color", self.r_color.as_ref(), color);
        members.put("g_color", self.g_color.as_ref(), color);
        members.put("b_color", self.b_color.as_ref(), color);
        members.put("a_color", self.a_color.as_ref(), color);
        members.finish(&self.extra)
    }
}

impl TextureSource {
    fn to_value(&self) -> Value {
        let mut members = Members::new();
        members.put("texture", self.texture.as_ref(), text);
        members.put("tint_map", self.tint_map.as_ref(), text);
        members.put("animated", self.animated.as_ref(), flag);
        members.put("frames", self.frames.as_ref(), int);
        members.put("use_face_uv", self.use_face_uv.as_ref(), flag);
        members.finish(&self.extra)
    }
}

impl GeometrySource {
    fn to_value(&self) -> Value {
        let mut members = Members::new();
        members.put("geometry", self.geometry.as_ref(), text);
        members.put("arm_size", self.arm_size.as_ref(), name);
        members.put("body_size", self.body_size.as_ref(), name);
        members.put("side", self.side.as_ref(), name);
        members.put("zone", self.zone.as_ref(), zones);
        members.put("is_face", self.is_face.as_ref(), face);
        members.put("texture", self.texture.as_ref(), text);
        members.put("tint_map", self.tint_map.as_ref(), text);
        members.put("animated", self.animated.as_ref(), flag);
        members.put("frames", self.frames.as_ref(), int);
        members.finish(&self.extra)
    }
}

impl AnimationSource {
    fn to_value(&self) -> Value {
        let mut members = Members::new();
        members.put("name", self.name.as_ref(), text);
        members.put("animationFile", self.animation_file.as_ref(), text);
        members.finish(&self.extra)
    }
}

impl PieceMetadata {
    /// The file as a JSON value: flat, or inside the envelope it was read from.
    // The members are written in the order the game reads them.
    pub(super) fn to_value(&self) -> Value {
        let mut members = Members::new();
        members.put("piece_id", self.piece_id.as_ref(), text);
        members.put("piece_name", self.piece_name.as_ref(), text);
        members.put("piece_type", self.piece_type.as_ref(), name);
        members.put("piece_sub_type", self.piece_sub_type.as_ref(), name);
        members.put("body_variant", self.body_variant.as_ref(), name);
        members.put("clothing_map", self.clothing_map.as_ref(), text);
        members.put("face_clothing_map", self.face_clothing_map.as_ref(), text);
        members.put("zone", self.zone.as_ref(), zones);
        members.put(
            "texture_sources",
            self.texture_sources.as_ref(),
            |sources| list(sources, TextureSource::to_value),
        );
        members.put(
            "geometry_sources",
            self.geometry_sources.as_ref(),
            |sources| list(sources, GeometrySource::to_value),
        );
        members.put(
            "animation_sources",
            self.animation_sources.as_ref(),
            |sources| list(sources, AnimationSource::to_value),
        );
        members.put("tint_color", self.tint_color.as_ref(), tint);
        members.put("tint_base_color", self.tint_base_color.as_ref(), tint);
        members.put(
            "allow_tint_override",
            self.allow_tint_override.as_ref(),
            flag,
        );
        let payload = members.finish(&self.extra);
        match &self.envelope {
            None => payload,
            Some(version) => {
                let mut document = VObject::new();
                document.insert("format_version", version.clone());
                document.insert(ENVELOPE, payload);
                document.into()
            }
        }
    }
}

impl PieceStoreInfo {
    /// The file as a JSON value.
    pub(super) fn to_value(&self) -> Value {
        let mut members = Members::new();
        members.put("piece_store_name", self.piece_store_name.as_ref(), text);
        members.put("piece_thumbnail", self.piece_thumbnail.as_ref(), text);
        members.finish(&self.extra)
    }
}
