//! `minecraft:aim_assist_categories`: aim-assist categories, the named sets of targeting
//! priorities that a preset's item settings select.

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::cereal::INVALID_NUMERIC_VALUE_TEXT;
use bedrock_addon_core::diagnostic::{Code, codes};
use bedrock_addon_core::id::AimAssistCategoryId;
use bedrock_addon_core::schema::{self, Schema};

/// What a priority below 0 is told as: cereal's "invalid numeric value"; one above 100 is the
/// range's own line (the bound's).
fn negative_priority() -> Code {
    codes::INVALID_VALUE_WORDED.with([INVALID_NUMERIC_VALUE_TEXT])
}

/// An aim-assist categories file (`minecraft:aim_assist_categories`, in
/// `aim_assist/categories/`), format 1.21.50.
///
/// JSON: `{"format_version": "1.21.50", "minecraft:aim_assist_categories": {"categories":
/// [{"name": "ns:name", "priorities": {…}}, …]}}`. The file loads all of its categories or
/// none: one name without a namespace rejects every category of the file.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet, Schema)]
#[doc(alias = "minecraft:aim_assist_categories")]
#[doc(alias = "aim_assist_categories")]
pub struct AimAssistCategories {
    /// The categories of the file, in order (JSON `categories`). An empty list loads nothing.
    pub categories: Vec<AimAssistCategory>,
}

/// One aim-assist category: the targeting
/// priorities a preset's item setting selects by name.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct AimAssistCategory {
    /// The category's name, `namespace:name` (JSON `name`).
    pub name: AimAssistCategoryId,
    /// The priorities (JSON `priorities`).
    pub priorities: Priorities,
}

/// The targeting priorities of a category (`priorities`): larger numbers win; each is 0 to 100.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct Priorities {
    /// The priority of each entity, by identifier (JSON `entities`).
    #[facet(schema::min = 0, schema::max = 100, schema::low_code = negative_priority())]
    pub entities: Option<IndexMap<String, i32>>,
    /// The priority of each block, by identifier (JSON `blocks`).
    #[facet(schema::min = 0, schema::max = 100, schema::low_code = negative_priority())]
    pub blocks: Option<IndexMap<String, i32>>,
    /// The priority of each block tag (JSON `block_tags`).
    #[facet(schema::min = 0, schema::max = 100, schema::low_code = negative_priority())]
    pub block_tags: Option<IndexMap<String, i32>>,
    /// The priority of each entity type family (JSON `entity_type_families`).
    #[facet(schema::min = 0, schema::max = 100, schema::low_code = negative_priority())]
    pub entity_type_families: Option<IndexMap<String, i32>>,
    /// The priority of an entity without an entry (JSON `entity_default`; see
    /// [`Priorities::entity_default`], 1 when the document leaves it out, what the game uses at
    /// run time).
    #[facet(
        schema::fallback = 1,
        schema::min = 0,
        schema::max = 100,
        schema::low_code = negative_priority()
    )]
    pub entity_default: Option<i32>,
    /// The priority of a block without an entry (JSON `block_default`; see
    /// [`Priorities::block_default`], 1 when the document leaves it out).
    #[facet(
        schema::fallback = 1,
        schema::min = 0,
        schema::max = 100,
        schema::low_code = negative_priority()
    )]
    pub block_default: Option<i32>,
}

impl AimAssistCategory {
    /// A category named `name` with `priorities`.
    #[must_use]
    pub fn new(name: AimAssistCategoryId, priorities: Priorities) -> Self {
        Self { name, priorities }
    }
}
