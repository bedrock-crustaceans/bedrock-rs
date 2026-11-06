//! `contents.json` read as a contents table: the plain fixtures and the per-asset cipher rows.

use super::{fixture, fixture_names};
use crate::contents::{AssetCipher, ContentsJson, ContentsTable};
use crate::{ContentKey, PackPath};

#[test]
fn every_plain_fixture_parses() {
    let names = fixture_names("contents_json");
    assert_eq!(names.len(), 6);
    for name in names {
        let contents = ContentsJson::parse(&fixture(&format!("contents_json/{name}")))
            .unwrap_or_else(|error| panic!("{name}: {error:?}"));
        assert!(!contents.content.is_empty(), "{name}");
        assert!(
            contents.content.iter().all(|entry| entry.key.is_none()),
            "{name}"
        );
        if name.starts_with("withversion") {
            assert_eq!(contents.version, Some(1), "{name}");
            assert!(contents.is_current(), "{name}");
        } else {
            assert_eq!(contents.version, None, "{name}");
            assert!(!contents.is_current(), "{name}");
        }
    }
}

#[test]
fn golden_small_fixture() {
    let contents = ContentsJson::parse(&fixture("contents_json/noversion_1.json")).unwrap();
    let paths: Vec<&str> = contents
        .content
        .iter()
        .map(|entry| entry.path.as_str())
        .collect();
    assert_eq!(paths, ["manifest.json", "entities/balloon.json"]);
}

#[test]
fn bom_and_comments_are_accepted() {
    let text = b"\xEF\xBB\xBF{ // keys\n \"version\": 1, \"content\": [] }";
    assert_eq!(ContentsJson::parse(text).unwrap().version, Some(1));
}

#[test]
fn nesting_deeper_than_128_is_refused() {
    let mut text = br#"{"content":[{"path":"a","x":"#.to_vec();
    text.extend(std::iter::repeat_n(b'[', 200));
    text.extend(std::iter::repeat_n(b']', 200));
    text.extend_from_slice(b"}]}");
    assert!(ContentsJson::parse(&text).is_err());
}

#[test]
fn syntax_errors_are_refused() {
    assert!(ContentsJson::parse(b"{\"content\": [").is_err());
    assert!(ContentsJson::parse(b"\x00\x13garbage").is_err());
}

#[test]
fn version_other_than_one_is_not_current() {
    let contents = ContentsJson::parse(br#"{"version": 0, "content": []}"#).unwrap();
    assert!(!contents.is_current());
}

mod table {
    use super::*;

    const KEY: &str = "0123456789abcdef0123456789ABCDEF";

    fn table(json: &str) -> ContentsTable {
        ContentsTable::from_json(&ContentsJson::parse(json.as_bytes()).unwrap())
    }

    fn key() -> ContentKey {
        ContentKey::from_slice(KEY.as_bytes()).unwrap()
    }

    #[test]
    fn entries_with_a_key_are_encrypted_cfb8_by_default() {
        let table = table(&format!(
            r#"{{"version":1,"content":[{{"path":"manifest.json"}},{{"path":"a.json","key":"{KEY}"}}]}}"#
        ));
        assert_eq!(table.get(&PackPath::from_static("manifest.json")), None);
        assert_eq!(
            table.get(&PackPath::from_static("a.json")),
            Some(&AssetCipher::Cfb8(key()))
        );
    }

    #[test]
    fn mode_selects_the_cipher() {
        let table = table(&format!(
            r#"{{"content":[{{"path":"cfb.json","key":"{KEY}","mode":"cfb"}},{{"path":"ctr.json","key":"{KEY}","mode":"ctr"}},{{"path":"odd.json","key":"{KEY}","mode":"ecb"}}]}}"#
        ));
        assert_eq!(
            table.get(&PackPath::from_static("cfb.json")),
            Some(&AssetCipher::Cfb8(key()))
        );
        assert_eq!(
            table.get(&PackPath::from_static("ctr.json")),
            Some(&AssetCipher::Ctr(key()))
        );
        assert_eq!(
            table.get(&PackPath::from_static("odd.json")),
            Some(&AssetCipher::Unusable)
        );
    }

    #[test]
    fn a_key_shorter_than_32_bytes_is_unusable() {
        let table = table(r#"{"content":[{"path":"a.json","key":"short"}]}"#);
        assert_eq!(
            table.get(&PackPath::from_static("a.json")),
            Some(&AssetCipher::Unusable)
        );
    }

    #[test]
    fn paths_normalise_and_rejected_paths_are_left_out() {
        let table = table(&format!(
            r#"{{"content":[{{"path":"textures\\a.png","key":"{KEY}"}},{{"path":"../b.json","key":"{KEY}"}}]}}"#
        ));
        assert!(
            table
                .get(&PackPath::from_static("textures/a.png"))
                .is_some()
        );
        assert_eq!(table.len(), 1);
    }
}
