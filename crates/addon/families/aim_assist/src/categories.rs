//! `minecraft:aim_assist_categories`: aim-assist categories, the named sets of targeting
//! priorities that a preset's item settings select.

use indexmap::IndexMap;

use bedrock_addon_core::id::AimAssistCategoryId;
use bedrock_addon_core::json::StrictInt;

/// The priority of a target without an entry and without a written default.
const DEFAULT_PRIORITY: i32 = 1;

/// An aim-assist categories file (`minecraft:aim_assist_categories`, in
/// `aim_assist/categories/`), format 1.21.50.
///
/// JSON: `{"format_version": "1.21.50", "minecraft:aim_assist_categories": {"categories":
/// [{"name": "ns:name", "priorities": {…}}, …]}}`. The file loads all of its categories or
/// none: one name without a namespace rejects every category of the file.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
#[doc(alias = "minecraft:aim_assist_categories")]
#[doc(alias = "aim_assist_categories")]
pub struct AimAssistCategories {
    /// The categories of the file, in order (JSON `categories`). An empty list loads nothing.
    pub categories: Vec<AimAssistCategory>,
}

/// One aim-assist category: the targeting
/// priorities a preset's item setting selects by name.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct AimAssistCategory {
    /// The category's name, `namespace:name` (JSON `name`).
    pub name: AimAssistCategoryId,
    /// The priorities (JSON `priorities`).
    pub priorities: Priorities,
}

/// The targeting priorities of a category (`priorities`): larger numbers win; each is 0 to 100.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
pub struct Priorities {
    /// The priority of each entity, by identifier (JSON `entities`).
    pub entities: Option<IndexMap<String, StrictInt<i32>>>,
    /// The priority of each block, by identifier (JSON `blocks`).
    pub blocks: Option<IndexMap<String, StrictInt<i32>>>,
    /// The priority of each block tag (JSON `block_tags`).
    pub block_tags: Option<IndexMap<String, StrictInt<i32>>>,
    /// The priority of each entity type family (JSON `entity_type_families`).
    pub entity_type_families: Option<IndexMap<String, StrictInt<i32>>>,
    /// The priority of an entity without an entry (JSON `entity_default`; see
    /// [`Priorities::entity_default`]).
    pub entity_default: Option<StrictInt<i32>>,
    /// The priority of a block without an entry (JSON `block_default`; see
    /// [`Priorities::block_default`]).
    pub block_default: Option<StrictInt<i32>>,
}

impl Priorities {
    /// [`entity_default`](Self::entity_default), 1 when the document leaves it out (what the
    /// game uses at run time).
    #[must_use]
    pub fn entity_default(&self) -> i32 {
        self.entity_default
            .map_or(DEFAULT_PRIORITY, |value| value.0)
    }

    /// [`block_default`](Self::block_default), 1 when the document leaves it out (what the
    /// game uses at run time).
    #[must_use]
    pub fn block_default(&self) -> i32 {
        self.block_default.map_or(DEFAULT_PRIORITY, |value| value.0)
    }
}

impl AimAssistCategory {
    /// A category named `name` with `priorities`.
    #[must_use]
    pub fn new(name: AimAssistCategoryId, priorities: Priorities) -> Self {
        Self { name, priorities }
    }
}
