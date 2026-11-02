//! Writing a side file back: the JSON it was read from.

use facet_value::{VArray, VObject};

use super::{
    AsepriteSheet, NINESLICE_KEYS, Nineslice, UiTextureInfo, floats, read_base_size,
    read_slice_size, slice_size_value,
};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::Value;

/// A side file: the members the game reads, then the others.
pub(super) fn texture_info_value(info: &UiTextureInfo) -> Value {
    match info {
        UiTextureInfo::Aseprite(sheet) => sheet_value(sheet),
        UiTextureInfo::Nineslice(nineslice) => nineslice_value(nineslice),
        UiTextureInfo::RejectedAseprite(written) => written.clone(),
    }
}

fn extend(object: &mut VObject, extra: &Extra) {
    for (key, value) in extra.iter() {
        if !object.contains_key(key) {
            object.insert(key, value.clone());
        }
    }
}

fn nineslice_value(nineslice: &Nineslice) -> Value {
    let mut root = VObject::new();
    // A member kept as written is written back while the typed field still is its reading.
    let mut put = |key: &str, typed: Option<Value>, unchanged: bool| {
        let value = match nineslice.extra.get(key) {
            Some(written) if unchanged => Some(written.clone()),
            _ => typed,
        };
        if let Some(value) = value {
            root.insert(key, value);
        }
    };
    let written = |key: &str| nineslice.extra.get(key);
    put(
        "nineslice_size",
        nineslice.nineslice_size.as_ref().map(slice_size_value),
        written("nineslice_size")
            .is_some_and(|size| read_slice_size(size) == nineslice.nineslice_size),
    );
    put(
        "base_size",
        nineslice.base_size.as_ref().map(|base| floats(base)),
        written("base_size").is_some_and(|size| read_base_size(size) == nineslice.base_size),
    );
    put(
        "tiled",
        nineslice.tiled.map(Value::from),
        written("tiled").is_some_and(|tiled| tiled.as_bool() == nineslice.tiled),
    );
    for (key, value) in nineslice.extra.iter() {
        if !NINESLICE_KEYS.contains(&key) {
            root.insert(key, value.clone());
        }
    }
    root.into_value()
}

fn sheet_value(sheet: &AsepriteSheet) -> Value {
    // A typed object with the unread members of the same object after its read ones.
    let nested = |read: VObject, extra: &Extra, key: &str| {
        let mut object = read;
        if let Some(others) = extra.get(key).and_then(Value::as_object) {
            for (name, value) in others {
                if !object.contains_key(name.as_str()) {
                    object.insert(name.as_str(), value.clone());
                }
            }
        }
        object
    };
    let mut frames = VArray::new();
    for frame in &sheet.frames {
        let mut position = VObject::new();
        position.insert("x", i64::from(frame.position[0]));
        position.insert("y", i64::from(frame.position[1]));
        let mut object = VObject::new();
        object.insert("frame", nested(position, &frame.extra, "frame"));
        object.insert("duration", i64::from(frame.duration));
        extend(&mut object, &frame.extra);
        frames.push(object);
    }
    let mut size = VObject::new();
    size.insert("w", i64::from(sheet.meta.size[0]));
    size.insert("h", i64::from(sheet.meta.size[1]));
    let mut meta = VObject::new();
    meta.insert("image", sheet.meta.image.as_str());
    meta.insert("size", nested(size, &sheet.meta.extra, "size"));
    extend(&mut meta, &sheet.meta.extra);
    let mut root = VObject::new();
    root.insert("frames", frames);
    root.insert("meta", meta);
    extend(&mut root, &sheet.extra);
    root.into_value()
}
