//! `PackWriter::add`: a document at its default path, refused when the path is taken.

use bedrock_addon::document::{Content, Placed};
use bedrock_addon::families::block::Block;
use bedrock_addon::families::block::component::DestructibleByMining;
use bedrock_addon::families::block_culling::CullingRules;
use bedrock_addon::id::{BlockId, CullingRulesId};
use bedrock_addon::json::EncodeError;
use bedrock_addon::pack::{PackWriter, WriteError};

use super::{bytes_at, listed, path, paths, ruby_manifest};

fn ruby_lamp() -> Block {
    let mut ruby = Block::new(BlockId::from_static("demo:ruby_lamp"));
    ruby.components.insert(DestructibleByMining::seconds(1.5));
    ruby
}

fn culling(identifier: &str) -> CullingRules {
    CullingRules::new(CullingRulesId::new(identifier).unwrap(), Vec::new())
}

#[test]
fn add_writes_a_document_at_its_default_path_and_returns_the_path() {
    let mut writer = PackWriter::new(ruby_manifest());
    let ruby = ruby_lamp();
    let at = writer.add(&ruby).unwrap();
    assert_eq!(at.as_str(), "blocks/demo/ruby_lamp.json");
    assert_eq!(at, ruby.default_path());
    assert_eq!(
        paths(&writer),
        ["manifest.json", "blocks/demo/ruby_lamp.json"]
    );
    assert_eq!(
        bytes_at(&writer, at.as_str()).unwrap(),
        ruby.to_json().unwrap().into_bytes(),
        "the text of `Content::write_json` in the default style"
    );
}

/// `minecraft:a/b` and `minecraft:a_b` of `block_culling` ask for one path. The second `add` is
/// refused; the first document stays as it was written (never last wins).
#[test]
fn add_refuses_a_default_path_that_is_taken_and_keeps_the_first_document() {
    let (slash, underscore) = (culling("minecraft:a/b"), culling("minecraft:a_b"));
    assert_eq!(slash.default_path(), underscore.default_path());
    for (first, second) in [(&slash, &underscore), (&underscore, &slash)] {
        let mut writer = PackWriter::new(ruby_manifest());
        let at = writer.add(first).unwrap();
        assert_eq!(at.as_str(), "block_culling/a_b.json");
        let before = listed(&writer);
        let refused = writer.add(second);
        assert!(
            matches!(refused, Err(WriteError::DuplicatePath { .. })),
            "{refused:?}"
        );
        assert_eq!(listed(&writer), before);
        assert_eq!(
            bytes_at(&writer, at.as_str()).unwrap(),
            first.to_json().unwrap().into_bytes()
        );
        // The author stores the second at a path of their choice.
        writer
            .put(path("block_culling/a_b_2.json"), second)
            .unwrap();
        assert_eq!(
            paths(&writer),
            [
                "manifest.json",
                "block_culling/a_b.json",
                "block_culling/a_b_2.json"
            ]
        );
    }
}

#[test]
fn add_refuses_a_path_taken_by_raw_bytes_and_takes_it_once_it_is_free() {
    let mut writer = PackWriter::new(ruby_manifest());
    let ruby = ruby_lamp();
    writer
        .put_bytes(ruby.default_path(), b"not a block".to_vec())
        .unwrap();
    let refused = writer.add(&ruby);
    assert!(
        matches!(&refused, Err(WriteError::DuplicatePath { path }) if path.as_str() == "blocks/demo/ruby_lamp.json"),
        "{refused:?}"
    );
    assert_eq!(
        bytes_at(&writer, "blocks/demo/ruby_lamp.json").unwrap(),
        b"not a block"
    );
    // The same document twice is refused as well: `add` never replaces.
    assert!(writer.remove(&ruby.default_path()));
    writer.add(&ruby).unwrap();
    assert!(matches!(
        writer.add(&ruby),
        Err(WriteError::DuplicatePath { .. })
    ));
}

#[test]
fn add_of_a_document_that_cannot_be_written_is_an_encode_error_and_changes_nothing() {
    let mut writer = PackWriter::new(ruby_manifest());
    let mut ruby = ruby_lamp();
    ruby.components
        .insert(DestructibleByMining::seconds(f32::NAN));
    let refused = writer.add(&ruby);
    assert!(
        matches!(
            refused,
            Err(WriteError::Encode(EncodeError::NotRepresentable { .. }))
        ),
        "{refused:?}"
    );
    assert_eq!(paths(&writer), ["manifest.json"]);
    // The path was not taken by the failed attempt.
    ruby.components.insert(DestructibleByMining::seconds(1.5));
    writer.add(&ruby).unwrap();
}

/// A loaded document is added with its unknown members, as `put` writes it.
#[test]
fn add_writes_a_document_with_its_unknown_members() {
    use bedrock_addon::load::LoadContext;
    use bedrock_addon::storage::PackPath;
    let text = r#"{"format_version": "1.16.0", "minecraft:block": {
        "description": {"identifier": "demo:ruby_lamp", "zz_description": 1},
        "components": {}
    }}"#;
    let document = Block::parse(
        &PackPath::from_static("blocks/x.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .expect("loads");
    assert!(!document.unknown().is_empty(), "the parse keeps a member");
    let mut writer = PackWriter::new(ruby_manifest());
    let at = writer.add(&document).unwrap();
    assert_eq!(at.as_str(), "blocks/demo/ruby_lamp.json");
    let written = String::from_utf8(bytes_at(&writer, at.as_str()).unwrap()).unwrap();
    assert!(written.contains("zz_description"), "{written}");
    assert_eq!(written, document.to_json().unwrap());
}
