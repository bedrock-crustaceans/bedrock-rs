//! Writing `skins.json` back.

use super::{Skin, SkinPack};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::Value;

impl Skin {
    pub(super) fn to_value(&self) -> Value {
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

    /// The fields as the JSON object they are written as.
    pub(super) fn fields_value(&self) -> Value {
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

    fn finish(mut self, extra: &UnknownMembers) -> Value {
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
