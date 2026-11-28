//! Writing `skins.json` back: the root as written while the fields still say it, else the
//! fields by core's generic writer (`schema::write_value`: the members in declaration order,
//! then the members kept as written whose key no member took).

use super::SkinPack;
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::{EncodeError, Value};

impl SkinPack {
    /// The file as it is written: the root as the author wrote it while the fields are still
    /// the game's reading of it, else the fields.
    pub(crate) fn to_value(&self) -> Result<Value, EncodeError> {
        if let Some(written) = self.written.as_ref() {
            let unchanged = Self::from_value(written, &mut Diagnostics::default()) == *self;
            if unchanged {
                return Ok(written.clone());
            }
        }
        self.fields_value()
    }

    /// The fields as the JSON object they are written as.
    pub(super) fn fields_value(&self) -> Result<Value, EncodeError> {
        bedrock_addon_core::schema::write_value(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::test_cost::assert_linear_at;

    /// The root kept beside the fields is left out of equality: a loaded pack that kept one
    /// equals the pack built by hand with the same fields, and is still written as authored.
    #[test]
    fn a_kept_root_is_left_out_of_equality() {
        let text = r#"{"serialize_name": "p", "localization_name": "p", "skins": [5]}"#;
        let root = bedrock_addon_core::json::from_str_jsonc(text).expect("JSON");
        let loaded = SkinPack::from_value(&root, &mut Diagnostics::default());
        assert_eq!(*loaded.written, Some(root.clone()));
        let built = SkinPack {
            serialize_name: Some("p".to_owned()),
            localization_name: Some("p".to_owned()),
            skins: Some(Vec::new()),
            ..SkinPack::default()
        };
        assert_eq!(loaded, built);
        assert_eq!(loaded.to_value().unwrap(), root);
        assert_ne!(built.to_value().unwrap(), root);
    }

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
            let root = bedrock_addon_core::json::from_str_jsonc(&text).expect("JSON");
            let pack = SkinPack::from_value(&root, &mut Diagnostics::default());
            assert!(pack.written.is_some(), "the root is not kept");
            pack
        };
        assert_linear_at("SkinPack::to_value", 2_000, build, |pack| {
            assert_eq!(Some(pack.to_value().unwrap()), *pack.written);
        });
    }
}
