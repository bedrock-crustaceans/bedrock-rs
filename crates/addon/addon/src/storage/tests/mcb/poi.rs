//! `minecraft:poi_block` and `minecraft:poi_tag` bodies at 1.26.40: the members in the
//! document's order, each written, defaults included.

use std::path::PathBuf;

use facet_value::Value;

use super::{Body, assert_unsupported, document};
use crate::storage::mcb::to_json_text;

const VERSION: (u16, u16, u32) = (1, 26, 40);

fn decode(
    key: &str,
    version: (u16, u16, u32),
    body: &Body,
) -> Result<String, crate::storage::StorageError> {
    to_json_text(&document(key, version, &body.0))
}

/// A type body: `demo:forge`, 2 tickets, distance 3, one block, one tag.
fn forge() -> Body {
    Body::default()
        .string("demo:forge")
        .raw(&[2, 3, 1])
        .string("minecraft:anvil")
        .raw(&[1])
        .string("demo:smithy")
}

#[test]
fn a_type_decodes_its_members_in_bind_order() {
    assert_eq!(
        decode("minecraft:poi_block", VERSION, &forge()).unwrap(),
        concat!(
            r#"{"format_version":"1.26.40","minecraft:poi_block":{"description":"#,
            r#"{"identifier":"demo:forge"},"tickets":2,"usable_distance":3,"#,
            r#""blocks":["minecraft:anvil"],"tags":["demo:smithy"]}}"#
        )
    );
}

#[test]
fn a_tag_decodes_its_members_in_bind_order() {
    let body = Body::default()
        .string("demo:town")
        .raw(&[1])
        .string("demo:job")
        .raw(&[0]);
    assert_eq!(
        decode("minecraft:poi_tag", VERSION, &body).unwrap(),
        concat!(
            r#"{"format_version":"1.26.40","minecraft:poi_tag":{"description":"#,
            r#"{"identifier":"demo:town"},"children":["demo:job"],"parents":[]}}"#
        )
    );
}

#[test]
fn a_poi_body_with_bytes_after_it_or_cut_short_or_at_another_version_is_unsupported() {
    assert_unsupported(decode("minecraft:poi_block", VERSION, &forge().raw(&[0])));
    let mut cut = forge();
    cut.0.pop();
    assert_unsupported(decode("minecraft:poi_block", VERSION, &cut));
    assert_unsupported(decode("minecraft:poi_block", (1, 26, 30), &forge()));
    assert_unsupported(decode(
        "minecraft:poi_tag",
        (1, 26, 50),
        &Body::default().string("a:b").raw(&[0, 0]),
    ));
}

/// `tests/fixtures/poi/<relative>`.
fn poi_fixture(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/poi")
        .join(relative)
}

#[test]
fn every_vanilla_poi_document_decodes_to_its_last_byte_as_its_decoded_twin() {
    let mut count = 0;
    for kind in ["blocks", "tags"] {
        let dir = poi_fixture(&format!("mcb/poi/{kind}"));
        let mut names: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        for name in names {
            let bytes = std::fs::read(dir.join(&name)).unwrap();
            let text = to_json_text(&bytes).unwrap_or_else(|e| panic!("{kind}/{name}: {e:?}"));
            let twin = std::fs::read_to_string(poi_fixture(&format!(
                "poi/poi42_experimental_poi/poi/{kind}/{name}"
            )))
            .unwrap();
            let decoded: Value = crate::json::from_str(&text).unwrap();
            let twin: Value = crate::json::from_str(&twin).unwrap();
            assert_eq!(decoded, twin, "{kind}/{name}");
            count += 1;
        }
    }
    assert_eq!(count, 20);
}
