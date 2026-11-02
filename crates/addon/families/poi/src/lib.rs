//! Points of interest: `Poi`, `PoiType`, `PoiTag`.
//!
//! Points of interest: the two documents of a behaviour pack's `poi/blocks` and `poi/tags`,
//! `minecraft:poi_block` (a POI type: the blocks that are one) and `minecraft:poi_tag` (a tag
//! and its place in the tag hierarchy), each one struct at format 1.26.40, loaded together as
//! the `Poi` group behind the `poi` experiment.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;
#[doc(hidden)]
pub use link::{Targets, link};

mod link;
mod load;
mod new;
mod parse;
mod resolve;
mod validate;

#[cfg(test)]
mod tests;

#[doc(hidden)]
pub use load::load_group;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::document::{Registry, Rejected};
use bedrock_addon_core::id::{BlockId, PoiTagId, PoiTypeId};
use bedrock_addon_core::json::StrictInt;

/// The points of interest of the behaviour packs (`bp.poi()`): the POI types and the tag
/// documents, loaded and resolved together.
///
/// A type is keyed by its `description.identifier`; the definition loaded last wins (the
/// stack is walked from the lowest pack up, so the highest pack's), and a block listed by two
/// types belongs to the one loaded last. The tag documents of one identifier are unioned.
/// Nothing is loaded without the `poi` experiment.
#[derive(Debug)]
#[non_exhaustive]
pub struct Poi {
    /// The POI types (`minecraft:poi_block`), by identifier.
    pub types: Registry<PoiType>,
    /// The tag documents (`minecraft:poi_tag`), by identifier.
    pub tags: Registry<PoiTag>,
    /// What the documents resolve to together: each block's type and the tag hierarchy.
    pub(crate) resolved: resolve::Resolved,
    /// The findings of the group itself: the experiment gate and the checks over every
    /// document.
    pub(crate) findings: Diagnostics,
}

impl Poi {
    /// The POI type `block` belongs to: the type loaded last among those listing it.
    #[must_use]
    pub fn type_of_block(&self, block: &BlockId) -> Option<&PoiTypeId> {
        self.resolved.blocks.get(block)
    }

    /// Every tag, in the order they came to exist: a tag has a document, or is named by a
    /// type's `tags`.
    pub fn tag_names(&self) -> impl Iterator<Item = &PoiTagId> + '_ {
        self.resolved.tags.keys()
    }

    /// Whether `tag` exists.
    #[must_use]
    pub fn contains_tag(&self, tag: &PoiTagId) -> bool {
        self.resolved.tags.contains_key(tag)
    }

    /// The types `tag` holds: the ones that name it, then the ones its narrower tags hold.
    /// Empty for a tag that does not exist.
    #[must_use]
    pub fn members(&self, tag: &PoiTagId) -> &[PoiTypeId] {
        self.resolved
            .tags
            .get(tag)
            .map_or(&[], |node| node.members.as_slice())
    }

    /// Whether `poi_type` is a member of `tag`, directly or through a narrower tag.
    #[must_use]
    pub fn has_tag(&self, poi_type: &PoiTypeId, tag: &PoiTagId) -> bool {
        self.members(tag).contains(poi_type)
    }

    /// The broader tags of `tag`: the ones it names in `parents` and the ones that name it in
    /// `children`, without the edges of a cycle and the ones to a tag that does not exist.
    #[must_use]
    pub fn parents(&self, tag: &PoiTagId) -> &[PoiTagId] {
        self.resolved
            .tags
            .get(tag)
            .map_or(&[], |node| node.parents.as_slice())
    }

    /// The narrower tags of `tag`: the mirror of [`parents`](Self::parents).
    #[must_use]
    pub fn children(&self, tag: &PoiTagId) -> &[PoiTagId] {
        self.resolved
            .tags
            .get(tag)
            .map_or(&[], |node| node.children.as_slice())
    }

    /// Every file that failed to load: the types' (`poi/blocks`), then the tags' (`poi/tags`).
    pub fn rejected(&self) -> impl Iterator<Item = &Rejected> + '_ {
        self.types.rejected().iter().chain(self.tags.rejected())
    }

    /// Every finding of the group: the types', the tags', then the group's own.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.types
            .diagnostics()
            .chain(self.tags.diagnostics())
            .chain(self.findings.iter())
    }
}

/// A point-of-interest type (`minecraft:poi_block`, in `poi/blocks/`): the blocks whose placed
/// instances are points of this type, how many entities may claim one and from how far it is
/// usable.
///
/// JSON: `{"format_version": "1.26.40", "minecraft:poi_block": {"description": {"identifier"},
/// "tickets", "usable_distance", "blocks", "tags"}}`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[doc(alias = "minecraft:poi_block")]
#[doc(alias = "poi_block")]
#[doc(alias = "TypeDocument")]
pub struct PoiType {
    /// The type's identifier (JSON `description`).
    pub description: PoiDescription,
    /// How many entities can reserve a point of this type, 0 to 255 (JSON `tickets`, default
    /// `0`: [`PoiType::tickets`]).
    #[facet(default)]
    pub tickets: Option<StrictInt<u8>>,
    /// The distance a point of this type is usable from, 0 to 255 (JSON `usable_distance`,
    /// default `1`: [`PoiType::usable_distance`]).
    #[facet(default)]
    pub usable_distance: Option<StrictInt<u8>>,
    /// The block types whose placed blocks are points of this type, every permutation of each
    /// (JSON `blocks`, each `namespace:name`).
    pub blocks: Vec<BlockId>,
    /// The tags the type carries (JSON `tags`, default none: [`PoiType::tags`]). A type is
    /// also a member of every broader tag of each.
    #[facet(default)]
    pub tags: Option<Vec<PoiTagId>>,
}

/// `minecraft:poi_block/description`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
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
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[doc(alias = "minecraft:poi_tag")]
#[doc(alias = "poi_tag")]
#[doc(alias = "TagDocument")]
pub struct PoiTag {
    /// The tag's identifier (JSON `description`).
    pub description: PoiTagDescription,
    /// The narrower tags, which inherit this tag's meaning (JSON `children`, default none:
    /// [`PoiTag::children`]).
    #[facet(default)]
    pub children: Option<Vec<PoiTagId>>,
    /// The broader tags this tag belongs to (JSON `parents`, default none:
    /// [`PoiTag::parents`]).
    #[facet(default)]
    pub parents: Option<Vec<PoiTagId>>,
}

/// `minecraft:poi_tag/description`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct PoiTagDescription {
    /// The tag's identifier, `namespace:name` of letters, digits and `_` (JSON `identifier`).
    pub identifier: PoiTagId,
}
