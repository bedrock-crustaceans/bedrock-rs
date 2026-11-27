//! The file read and written without a stack: the vanilla file and two fixtures round trip, and a
//! file that is no array is rejected on its own.

use bedrock_addon_core::document::{Content, Placed};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_flipbooks::Flipbooks;

const FAMILY: &str = "flipbook_textures";
const FILE: &str = "textures/flipbook_textures.json";

/// The bytes of `file` of fixture `name`.
fn fixture_file(name: &str, file: &str) -> Vec<u8> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../addon/tests/fixtures/texture")
        .join(FAMILY)
        .join(name)
        .join(file);
    std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn parse(bytes: &[u8]) -> Flipbooks {
    let path = PackPath::new(FILE).unwrap();
    Flipbooks::parse(&path, bytes, &LoadContext::detached())
        .unwrap_or_else(|rejected| panic!("{:?}", rejected.diagnostics))
        .into_value()
}

#[test]
fn flipbooks_are_written_back_to_the_same_flipbooks() {
    for fixture in ["m00_vanilla_1_26_50", "p06_unknown_member", "p00_defaults"] {
        let bytes = fixture_file(fixture, &format!("packs/0/{FILE}"));
        let first = parse(&bytes);
        let written = first.to_json().unwrap();
        let second = parse(written.as_bytes());
        assert_eq!(first, second, "{fixture}");
        assert_eq!(
            second.to_json().unwrap(),
            written,
            "{fixture}: the rewrite differs"
        );
    }
}

#[test]
fn a_member_the_entry_does_not_write_is_not_written() {
    let flipbooks = parse(br#"[ { "atlas_tile": "x" } ]"#);
    assert_eq!(
        flipbooks.to_json().unwrap(),
        "[\n  {\n    \"atlas_tile\": \"x\"\n  }\n]\n"
    );
    assert_eq!(flipbooks.default_path().as_str(), FILE);
}

#[test]
fn a_file_that_is_no_array_is_rejected_on_its_own() {
    let path = PackPath::new(FILE).unwrap();
    let rejected = Flipbooks::parse(&path, b"{}", &LoadContext::detached()).unwrap_err();
    assert_eq!(
        rejected.diagnostics.iter().next().unwrap().code.id(),
        "core/invalid_value"
    );
    let rejected = Flipbooks::parse(&path, b"[", &LoadContext::detached()).unwrap_err();
    assert_eq!(
        rejected.diagnostics.iter().next().unwrap().code.id(),
        "core/syntax"
    );
}
