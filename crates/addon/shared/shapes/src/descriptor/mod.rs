//! The item and block descriptors: how a document names an item or a block, with the
//! findings vanilla's readers log for a descriptor of the wrong shape.

mod block;
mod item;

pub use block::{BlockDescriptor, BlockSpecifier, StateValue};
pub use item::{ItemDescriptor, ItemDescriptorForm};

use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::schema::{SchemaCx, SchemaFields};

/// An item descriptor's `tags` (the tags form) is a Molang field, under the walk's default
/// spec, as a block descriptor's is.
impl MolangFields for ItemDescriptor {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        if let Self::Tags { tags, .. } = self {
            visitor.field_default("tags", tags);
        }
    }
}

/// A block descriptor's `tags` (the tags and mixed forms) is a Molang field, under the walk's
/// default spec: a family whose descriptor tags read otherwise (`DESCRIPTOR_TAGS` in feature,
/// block and entity) walks them with its own function, named by `schema::molang_with`.
impl MolangFields for BlockDescriptor {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        if let Self::Tags { tags, .. } | Self::Mixed { tags, .. } = self {
            visitor.field_default("tags", tags);
        }
    }
}

// A block specifier holds no Molang.
bedrock_addon_core::impl_no_molang!(BlockSpecifier);

// An item descriptor and a block specifier read their own syntax and hold no bound the derive
// checks and no trigger (an item descriptor's `tags` is Molang, walked by `MolangFields`).
bedrock_addon_core::impl_schema_leaf!(ItemDescriptor, BlockSpecifier);

/// A block descriptor reads its own syntax and holds no bound and no trigger; its `tags` (the
/// tags and mixed forms) is walked at its member, a Molang value as any other (the Molang walk
/// is `MolangFields`'s).
impl SchemaFields for BlockDescriptor {
    fn check_fields(&mut self, cx: &mut SchemaCx<'_>) {
        if let Self::Tags { tags, .. } | Self::Mixed { tags, .. } = self {
            cx.enter("tags");
            tags.check_fields(cx);
            cx.leave();
        }
    }

    fn trigger_members(_: &str, _: &mut Vec<String>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::diagnostic::JsonPath;
    use bedrock_addon_core::molang::{MolangFieldSpec, visit_fields_under};

    /// The path and spec of every field the walk of `descriptor` visits, under `default_spec`.
    fn fields_of(
        descriptor: &mut BlockDescriptor,
        default_spec: &MolangFieldSpec,
    ) -> Vec<(String, MolangFieldSpec)> {
        let mut found = Vec::new();
        visit_fields_under(
            descriptor,
            &JsonPath::new().child("block"),
            default_spec,
            |field| found.push((field.path.to_string(), *field.spec)),
        );
        found
    }

    #[test]
    fn a_block_descriptor_visits_its_tags_under_the_walks_default_spec() {
        let spec = MolangFieldSpec::KEPT.tags();
        let json = |text: &str| {
            bedrock_addon_core::json::from_str_jsonc::<bedrock_addon_core::json::Value>(text)
                .expect("JSON")
        };
        let mut tags = BlockDescriptor::from(json(r#"{"tags": "query.any_tag('stone')"}"#));
        assert_eq!(
            fields_of(&mut tags, &spec),
            [("block/tags".to_owned(), spec)]
        );
        let mut mixed = BlockDescriptor::from(json(
            r#"{"name": "minecraft:stone", "tags": "query.any_tag('stone')"}"#,
        ));
        assert_eq!(
            fields_of(&mut mixed, &spec),
            [("block/tags".to_owned(), spec)]
        );
        let mut block = BlockDescriptor::from(json(r#"{"name": "minecraft:stone"}"#));
        assert_eq!(fields_of(&mut block, &spec), []);
    }
}
