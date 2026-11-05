//! Write-back as authored: what `to_json` writes for a file that loads is the file,
//! as JSON values, whatever the game makes of its members. The documents are the review's
//! probes: wrong-typed members, containers of another kind, skipped and empty entries, roots of
//! another kind.

use bedrock_addon::document::Content;
use bedrock_addon::families::font::{EmoticonList, FontMetadata};
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;

use super::common::{as_authored, json};

#[test]
fn font_lists_are_written_as_authored() {
    as_authored::<FontMetadata>(&[
        r#"{"version":1}"#,
        r#"{"version":1,"fonts":[{"font_format":"ttf","font_name":"A","font_file":"font/a"}]}"#,
        r#"{"version":1,"fonts":[{"font_format":"ttf","font_name":"A","font_file":5,"version":"2","target_font_render_size":1.5,"lowPerformanceCompatible":"false","generate_pages_on_start":{"first_page":0,"last_page":1}}, {"font_format":5,"font_name":"B"}, 3],"font_aliases":[{"alias":"X","fonts":[{"font_reference":"A","font_scale":1,"font_language_code":3,"font_ranges":5}],"font_scale_factor_override":2},{"alias":"Y","fonts":[]},{"alias":5}]}"#,
        r#"{"version":1,"fonts":{"k":{"font_format":"ttf","font_name":"A","font_file":"f"}},"font_aliases":7}"#,
        r#"{"version":1,"font_aliases":7}"#,
        r#"{"version":1,"fonts":"x","font_aliases":{"b":{"alias":"B","fonts":[{"font_reference":"UIFont"}]}}}"#,
    ]);
}

#[test]
fn emoticon_lists_are_written_as_authored() {
    as_authored::<EmoticonList>(&[
        "[]",
        r#"[{"name":"a","code":"0xE000"}]"#,
        r#"[{"name":5,"code":"0xE000"}, 3, {"name":"b","code":7,"k":1}, {"name":"c","code":"zz","k":2}]"#,
        "{}",
        r#"{"name":"a"}"#,
        "7",
    ]);
}

/// The kept root is written only while the fields are still the game's reading of it: a view
/// that was changed writes itself.
#[test]
fn a_changed_view_writes_itself() {
    let path = PackPath::from_static("x.json");
    let text = r#"[{"name":"a","code":"0xE000"}, 3]"#;
    let document = EmoticonList::parse(&path, text.as_bytes(), &LoadContext::detached()).unwrap();
    let mut list = document.into_value();
    assert_eq!(*list.written, Some(json(text)));
    list.emoticons[0].name = "b".to_owned();
    assert_eq!(
        json(&list.to_json().unwrap()),
        json(r#"[{"name":"b","code":"0xE000"}]"#)
    );
    // A file the fields say whole keeps nothing beside them.
    let text = r#"[{"name":"a","code":"0xE000"}]"#;
    let document = EmoticonList::parse(&path, text.as_bytes(), &LoadContext::detached()).unwrap();
    assert_eq!(*document.written, None);
}
