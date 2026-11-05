//! The two POI documents, their constructors and the defaults of members a document may leave
//! out.

use bedrock_addon_core::id::{BlockId, PoiTagId, PoiTypeId};
use bedrock_addon_core::schema::{self, Schema};

/// A point-of-interest type (`minecraft:poi_block`, in `poi/blocks/`): the blocks whose placed
/// instances are points of this type, how many entities may claim one and from how far it is
/// usable.
///
/// JSON: `{"format_version": "1.26.40", "minecraft:poi_block": {"description": {"identifier"},
/// "tickets", "usable_distance", "blocks", "tags"}}`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[doc(alias = "minecraft:poi_block")]
#[doc(alias = "poi_block")]
#[doc(alias = "TypeDocument")]
pub struct PoiType {
    /// The type's identifier (JSON `description`).
    pub description: PoiDescription,
    /// How many entities can reserve a point of this type, 0 to 255 (JSON `tickets`, default
    /// `0`: [`PoiType::tickets`]).
    #[facet(schema::fallback = 0)]
    pub tickets: Option<u8>,
    /// The distance a point of this type is usable from, 0 to 255 (JSON `usable_distance`,
    /// default `1`: [`PoiType::usable_distance`]).
    #[facet(schema::fallback = 1)]
    pub usable_distance: Option<u8>,
    /// The block types whose placed blocks are points of this type, every permutation of each
    /// (JSON `blocks`, each `namespace:name`).
    pub blocks: Vec<BlockId>,
    /// The tags the type carries (JSON `tags`, default none: [`PoiType::tags`]). A type is
    /// also a member of every broader tag of each.
    pub tags: Option<Vec<PoiTagId>>,
}

/// `minecraft:poi_block/description`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct PoiDescription {
    /// The type's identifier, `namespace:name` of letters, digits and `_` (JSON
    /// `identifier`).
    pub identifier: PoiTypeId,
}

/// A point-of-interest tag (`minecraft:poi_tag`, in `poi/tags/`): a name types carry, and its
/// broader and narrower tags.
///
/// JSON: `{"format_version": "1.26.40", "minecraft:poi_tag": {"description": {"identifier"},
/// "children", "parents"}}`. `parents` on one tag and `children` on the other say the same
/// thing; a tag holds the types of its narrower tags.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[doc(alias = "minecraft:poi_tag")]
#[doc(alias = "poi_tag")]
#[doc(alias = "TagDocument")]
pub struct PoiTag {
    /// The tag's identifier (JSON `description`).
    pub description: PoiTagDescription,
    /// The narrower tags, which inherit this tag's meaning (JSON `children`, default none:
    /// [`PoiTag::children`]).
    pub children: Option<Vec<PoiTagId>>,
    /// The broader tags this tag belongs to (JSON `parents`, default none:
    /// [`PoiTag::parents`]).
    pub parents: Option<Vec<PoiTagId>>,
}

/// `minecraft:poi_tag/description`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct PoiTagDescription {
    /// The tag's identifier, `namespace:name` of letters, digits and `_` (JSON `identifier`).
    pub identifier: PoiTagId,
}

impl PoiType {
    /// A type of `blocks` with every other member left out (its default).
    pub fn new(identifier: PoiTypeId, blocks: Vec<BlockId>) -> PoiType {
        PoiType {
            description: PoiDescription { identifier },
            tickets: None,
            usable_distance: None,
            blocks,
            tags: None,
        }
    }

    /// The tags the type carries: `tags`, or none when left out.
    pub fn tags(&self) -> &[PoiTagId] {
        self.tags.as_deref().unwrap_or_default()
    }
}

impl PoiTag {
    /// A tag without broader or narrower tags.
    pub fn new(identifier: PoiTagId) -> PoiTag {
        PoiTag {
            description: PoiTagDescription { identifier },
            children: None,
            parents: None,
        }
    }

    /// The narrower tags: `children`, or none when left out.
    pub fn children(&self) -> &[PoiTagId] {
        self.children.as_deref().unwrap_or_default()
    }

    /// The broader tags: `parents`, or none when left out.
    pub fn parents(&self) -> &[PoiTagId] {
        self.parents.as_deref().unwrap_or_default()
    }
}
