//! Skipped fields: a document member named like a `#[facet(skip)]` field is an unknown member
//! like any other, reported and kept for write-back; the reader binds no member to a skipped
//! field.

#[cfg(feature = "item")]
mod item {
    use bedrock_addon::document::Content;
    use bedrock_addon::families::item::Item;
    use bedrock_addon::load::LoadContext;
    use bedrock_addon::storage::PackPath;

    /// A member named `extra` in the item object and in its description loads as an unknown
    /// member, warned and kept for write-back (at 1.21.90: from 1.26.30 the strict slice
    /// refuses an unknown member).
    #[test]
    fn a_member_named_extra_is_an_unrecognized_field() {
        const TEXT: &str = r#"{"format_version": "1.21.90", "minecraft:item": {"description": {"identifier": "demo:x", "extra": {"a": 1}}, "components": {"minecraft:max_stack_size": 16}, "extra": 1}}"#;
        let document = Item::parse(
            &PackPath::from_static("items/x.json"),
            TEXT.as_bytes(),
            &LoadContext::detached(),
        )
        .unwrap_or_else(|rejected| panic!("rejected: {:#?}", rejected.diagnostics));
        let mut unrecognized: Vec<String> = document
            .diagnostics()
            .iter()
            .filter(|f| matches!(f.code.id(), "core/unrecognized_field"))
            .map(|f| f.location.json_path.to_string())
            .collect();
        unrecognized.sort();
        assert_eq!(
            unrecognized,
            ["minecraft:item/description/extra", "minecraft:item/extra"],
            "{:#?}",
            document.diagnostics()
        );
    }
}
