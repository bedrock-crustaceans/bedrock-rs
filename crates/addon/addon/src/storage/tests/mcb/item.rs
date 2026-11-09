//! `minecraft:item` bodies through the item family's reader.

/// The items the game bakes after 1.26.30: each decodes, loads from a base-game pack as an MCB
/// source, and is written back with its payload unchanged. The three at 1.26.50 are parsed by the newest struct and not upgraded;
/// the two at 1.26.40 select the 1.26.30 struct and run the step, an identity on them; the
/// 1.26.30 `trial_key` runs it too.
mod baked_items_load {
    use bedrock_addon_core::document::Content;
    use bedrock_addon_core::load::{DocumentSource, LoadContext};
    use bedrock_addon_core::pack_identity::PackPath;
    use bedrock_addon_core::version::FormatVersion;

    use crate::families::item::Item;
    use crate::json::Value;
    use crate::storage::mcb::to_json_text;

    /// The JSON text of the MCB fixture `name` (`crates/addon/storage/tests/fixtures/mcb/`).
    fn decoded(name: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../storage/tests/fixtures/mcb")
            .join(format!("{name}.json"));
        let bytes = std::fs::read(&path).unwrap();
        to_json_text(&bytes).unwrap()
    }

    fn base_game_mcb() -> LoadContext {
        let mut cx = LoadContext::detached();
        cx.is_base_game_pack = true;
        cx.source = DocumentSource::Mcb;
        cx
    }

    fn payload(text: &str) -> Value {
        let value: Value = crate::json::from_str(text).unwrap();
        value.as_object().unwrap()["minecraft:item"].clone()
    }

    #[test]
    fn the_baked_vanilla_items_load_and_write_back_their_payload() {
        for (name, parsed_as) in [
            ("red_shrub_1_26_50", FormatVersion::new(1, 26, 50)),
            ("shelf_mushroom_1_26_50", FormatVersion::new(1, 26, 50)),
            ("ice_ball_1_26_50", FormatVersion::new(1, 26, 50)),
            ("ice_crystal_1_26_40", FormatVersion::new(1, 26, 30)),
            ("icicle_1_26_40", FormatVersion::new(1, 26, 30)),
            ("trial_key", FormatVersion::new(1, 26, 30)),
        ] {
            let text = decoded(name);
            let document = Item::parse(
                &PackPath::from_static("items/x.json"),
                text.as_bytes(),
                &base_game_mcb(),
            )
            .unwrap_or_else(|rejected| panic!("{name}: {:#?}", rejected.diagnostics));
            assert!(
                document.diagnostics().is_empty(),
                "{name}: {:#?}",
                document.diagnostics()
            );
            assert_eq!(document.version().parsed_as, parsed_as, "{name}");
            assert_eq!(
                document.version().upgraded,
                parsed_as != FormatVersion::new(1, 26, 50),
                "{name}"
            );
            let written = document.to_json().unwrap();
            assert_eq!(payload(&written), payload(&text), "{name}: {written}");
            assert!(
                written.contains("\"format_version\": \"1.26.50\""),
                "{name}: {written}"
            );
        }
    }
}
