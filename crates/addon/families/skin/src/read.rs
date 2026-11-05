//! Reading and writing `skins.json`.

use super::{Skin, SkinPack};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, VanillaText,
};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::Value;

/// The longest path the game's fixed buffers hold.
pub(super) const MAX_PATH: usize = 1023;
/// The most skins creator tools allow in a pack.
const MAX_SKINS: usize = 80;

/// Vanilla's content-log text for an entry of `skins` that is no object.
const ENTRY_NOT_OBJECT: &str = "Invalid entry in skins.json - Every entry under 'skins' must be a json object - it has to start with an \"{\" and end with a \"}\"";

const PACK_KEYS: [&str; 3] = ["serialize_name", "localization_name", "skins"];
const STRING_KEYS: [&str; 5] = ["localization_name", "geometry", "texture", "type", "cape"];
const PATCH_KEYS: [&str; 4] = [
    "animations",
    "enable_attachables",
    "held_item_ignores_lighting",
    "hide_armor",
];

/// A finding of the crate's own, the game being silent about it.
fn note(at: JsonPath) -> Diagnostic {
    Diagnostic::info(DiagnosticCode::InvalidValue { text: None }).at(at)
}

fn key(at: &JsonPath, name: &str) -> JsonPath {
    let mut path = at.clone();
    path.push(name);
    path
}

/// The members of one object: a string member as its typed reading, a member of another
/// type kept as written, every member not read kept and reported.
struct Fields<'v> {
    object: Option<&'v facet_value::VObject>,
    at: JsonPath,
    extra: Extra,
}

impl<'v> Fields<'v> {
    fn new(value: &'v Value, at: JsonPath) -> Self {
        Self {
            object: value.as_object(),
            at,
            extra: Extra::default(),
        }
    }

    fn raw(&self, name: &str) -> Option<&'v Value> {
        self.object?.get(name)
    }

    /// The string member `name`; a member of another type goes to `extra` as written.
    fn string(&mut self, name: &str) -> Option<String> {
        let member = self.raw(name)?;
        if let Some(text) = member.as_string() {
            Some(text.as_str().to_owned())
        } else {
            self.extra.insert(name.to_owned(), member.clone());
            None
        }
    }

    /// `extra`, with every member that is none of `read` added and reported.
    fn finish(mut self, read: &[&str], diagnostics: &mut Diagnostics) -> Extra {
        for (name, member) in self.object.into_iter().flatten() {
            if !read.contains(&name.as_str()) {
                diagnostics.push(
                    Diagnostic::info(DiagnosticCode::UnrecognizedField { text: None })
                        .at(key(&self.at, name.as_str())),
                );
                self.extra.insert(name.as_str().to_owned(), member.clone());
            }
        }
        self.extra
    }
}

impl Skin {
    fn read(element: &Value, at: &JsonPath, diagnostics: &mut Diagnostics) -> Self {
        let mut fields = Fields::new(element, at.clone());
        let mut skin = Self {
            localization_name: fields.string("localization_name"),
            geometry: fields.string("geometry"),
            texture: fields.string("texture"),
            type_: fields.string("type"),
            cape: fields.string("cape"),
            animations: fields.raw("animations").cloned(),
            enable_attachables: fields.raw("enable_attachables").cloned(),
            held_item_ignores_lighting: fields.raw("held_item_ignores_lighting").cloned(),
            hide_armor: fields.raw("hide_armor").cloned(),
            extra: Extra::default(),
        };
        // A path the game's buffer cannot hold reads as empty; the game's only word is a debug
        // assert, no content-log line, so the crate reports an information.
        for (name, path) in [("texture", &skin.texture), ("cape", &skin.cape)] {
            if path.as_ref().is_some_and(|path| path.len() > MAX_PATH) {
                diagnostics.push(note(key(at, name)));
            }
        }
        // The game compares the type with nothing and logs nothing; creators mean `free` or
        // `paid`, so the crate reports an information.
        if fields.raw("type").is_some() && !matches!(skin.type_.as_deref(), Some("free" | "paid")) {
            diagnostics.push(note(key(at, "type")));
        }
        let read: Vec<&str> = STRING_KEYS.into_iter().chain(PATCH_KEYS).collect();
        skin.extra = fields.finish(&read, diagnostics);
        skin
    }

    fn to_value(&self) -> Value {
        let text = |text: &Option<String>| text.as_deref().map(Value::from);
        let mut object = Written::default();
        object.put("localization_name", text(&self.localization_name));
        object.put("geometry", text(&self.geometry));
        object.put("texture", text(&self.texture));
        object.put("type", text(&self.type_));
        object.put("cape", text(&self.cape));
        object.put("animations", self.animations.clone());
        object.put("enable_attachables", self.enable_attachables.clone());
        object.put(
            "held_item_ignores_lighting",
            self.held_item_ignores_lighting.clone(),
        );
        object.put("hide_armor", self.hide_armor.clone());
        object.finish(&self.extra)
    }
}

impl SkinPack {
    /// Reads the root object of a `skins.json`, and keeps it as written when the fields would
    /// not write it back.
    pub(crate) fn from_value(root: &Value, diagnostics: &mut Diagnostics) -> Self {
        Self::from_value_placed(root, diagnostics).0
    }

    /// [`from_value`](Self::from_value), with the place in the file of each skin held.
    pub(super) fn from_value_placed(
        root: &Value,
        diagnostics: &mut Diagnostics,
    ) -> (Self, Vec<JsonPath>) {
        let (mut pack, places) = Self::read(root, diagnostics);
        if pack.fields_value() != *root {
            pack.written = Some(root.clone());
        }
        (pack, places)
    }

    /// The file as it is written: the root as the author wrote it while the fields are still
    /// the game's reading of it, else the fields.
    pub(crate) fn to_value(&self) -> Value {
        if let Some(written) = &self.written {
            let unchanged = Self::from_value(written, &mut Diagnostics::default()) == *self;
            if unchanged {
                return written.clone();
            }
        }
        self.fields_value()
    }

    /// The game's reading of the root object, with nothing kept beside it, and the place in
    /// the file of each skin held.
    fn read(root: &Value, diagnostics: &mut Diagnostics) -> (Self, Vec<JsonPath>) {
        let at = JsonPath::new();
        let mut fields = Fields::new(root, at.clone());
        let serialize_name = fields.string("serialize_name");
        let localization_name = fields.string("localization_name");
        let skins_at = key(&at, "skins");
        let mut places = Vec::new();
        let skins = fields.raw("skins").map(|member| {
            // The game sizes the pack by an array and then iterates whatever is there: an
            // object gives its member values in key order, a scalar nothing.
            let mut entries: Vec<(JsonPath, &Value)> = Vec::new();
            if let Some(elements) = member.as_array() {
                for (i, element) in elements.iter().enumerate() {
                    let mut at = skins_at.clone();
                    at.push_index(i);
                    entries.push((at, element));
                }
            } else {
                // The game iterates the members without a line, so the crate reports an
                // information.
                diagnostics.push(note(skins_at.clone()));
                let mut members: Vec<(&str, &Value)> = member
                    .as_object()
                    .into_iter()
                    .flatten()
                    .map(|(name, value)| (name.as_str(), value))
                    .collect();
                members.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
                entries.extend(
                    members
                        .into_iter()
                        .map(|(name, value)| (key(&skins_at, name), value)),
                );
            }
            let mut skins = Vec::with_capacity(entries.len());
            for (at, element) in entries {
                if element.is_object() {
                    skins.push(Skin::read(element, &at, diagnostics));
                    places.push(at);
                } else {
                    // The game logs the line at the content log's error level and goes on to the
                    // next entry: a kept error.
                    let code = DiagnosticCode::InvalidType {
                        text: Some(VanillaText::Fixed(ENTRY_NOT_OBJECT)),
                    };
                    diagnostics.push(Diagnostic::error(code).at(at).into_kept());
                }
            }
            skins
        });
        // Creator tools want the two names equal; the game reads only one.
        if serialize_name.is_none() || serialize_name != localization_name {
            diagnostics.push(
                Diagnostic::info(DiagnosticCode::SkinPackNameMismatch)
                    .at(key(&at, "serialize_name")),
            );
        }
        // A creator rule, not the game's.
        if skins.as_ref().is_some_and(|skins| skins.len() > MAX_SKINS) {
            diagnostics.push(Diagnostic::info(DiagnosticCode::SkinPackCreatorRule).at(skins_at));
        }
        let pack = Self {
            serialize_name,
            localization_name,
            skins,
            extra: fields.finish(&PACK_KEYS, diagnostics),
            written: None,
        };
        (pack, places)
    }

    /// The fields as the JSON object they are written as.
    fn fields_value(&self) -> Value {
        let text = |text: &Option<String>| text.as_deref().map(Value::from);
        let mut object = Written::default();
        object.put("serialize_name", text(&self.serialize_name));
        object.put("localization_name", text(&self.localization_name));
        object.put(
            "skins",
            self.skins.as_ref().map(|skins| {
                let mut array = facet_value::VArray::new();
                for skin in skins {
                    array.push(skin.to_value());
                }
                array.into_value()
            }),
        );
        object.finish(&self.extra)
    }
}

/// An object being written: the typed members, then the members kept as written whose key no
/// typed member wrote.
#[derive(Default)]
struct Written {
    object: facet_value::VObject,
}

impl Written {
    fn put(&mut self, name: &str, value: Option<Value>) {
        if let Some(value) = value {
            self.object.insert(name, value);
        }
    }

    fn finish(mut self, extra: &Extra) -> Value {
        for (name, value) in extra.iter() {
            if !self.object.contains_key(name) {
                self.object.insert(name, value.clone());
            }
        }
        self.object.into_value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::test_cost::assert_linear_at;

    /// A write of a skin pack that keeps its root reads the kept root again and compares
    /// (linear in the skins).
    #[test]
    fn a_kept_skin_pack_is_written_in_linear_time() {
        let build = |entries: usize| {
            let skins: Vec<String> = (0..entries)
                .map(|i| {
                    format!(
                        r#"{{ "localization_name": "s{i}", "geometry": "geometry.s{i}", "texture": "s{i}.png", "type": "free" }}"#
                    )
                })
                .collect();
            // The last entry is no object: the game skips it, the root is kept.
            let text = format!(
                r#"{{ "serialize_name": "p", "localization_name": "p", "skins": [{}, 5] }}"#,
                skins.join(",")
            );
            let root = bedrock_addon_core::json::from_str::<Value>(&text).expect("JSON");
            let pack = SkinPack::from_value(&root, &mut Diagnostics::default());
            assert!(pack.written.is_some(), "the root is not kept");
            pack
        };
        assert_linear_at("SkinPack::to_value", 2_000, build, |pack| {
            assert_eq!(Some(pack.to_value()), pack.written);
        });
    }
}
