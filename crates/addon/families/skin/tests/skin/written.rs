//! The written text of `skins.json`, pinned in `skin/written.pins`: what `to_json` writes for
//! probes of each member, a root kept as written and a root the fields say, and after an edit
//! of the typed value (`bedrock_addon_test_support::written_pins`).

use bedrock_addon_family_skin::SkinPack;
use bedrock_addon_test_support::written_pins::{assert_as_pinned, edited, render, written};

const PATH: &str = "skins.json";

const PROBES: &[&str] = &[
    r#"{"serialize_name": "p", "localization_name": "p", "skins": [{"localization_name": "a", "geometry": "geometry.humanoid.custom", "texture": "a.png", "type": "free", "cape": "c.png", "animations": {"move.arms": "animation.x"}, "enable_attachables": false, "held_item_ignores_lighting": [1], "hide_armor": "yes", "tag": 7}]}"#,
    r#"{"serialize_name": 5, "localization_name": true, "extra": 1, "skins": [{"localization_name": false, "geometry": 3, "texture": null, "cape": true, "type": []}]}"#,
    r#"{"skins": [{"localization_name": "a", "texture": "a.png"}, "x", 4]}"#,
    r#"{"skins": {"b": {"texture": "b.png"}, "a": {"texture": "a.png"}}}"#,
    r#"{"skins": [{"texture": "a.png", "hide_armor": null, "animations": null}], "odd": [null]}"#,
    r#"{"skins": 5}"#,
    r#"{"skins": []}"#,
    "{}",
];

#[test]
fn the_skin_pack_is_written_as_pinned() {
    let mut cases: Vec<(String, String)> = Vec::new();
    for probe in PROBES {
        cases.push(((*probe).to_owned(), written::<SkinPack>(PATH, probe)));
    }
    for probe in PROBES {
        cases.push((
            format!("{probe}, localization_name set to \"edited\""),
            edited::<SkinPack>(PATH, probe, |pack| {
                pack.localization_name = Some("edited".to_owned());
            }),
        ));
    }
    assert_as_pinned(
        env!("CARGO_MANIFEST_DIR"),
        "skin/written.pins",
        &render(&cases),
    );
}
