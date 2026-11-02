//! Constructors and the defaults of members a document may leave out.

use super::{PoiDescription, PoiTag, PoiTagDescription, PoiType};
use bedrock_addon_core::id::{BlockId, PoiTagId, PoiTypeId};

impl PoiType {
    /// A type of `blocks` with every other member left out (its default).
    #[must_use]
    pub fn new(identifier: PoiTypeId, blocks: Vec<BlockId>) -> PoiType {
        PoiType {
            description: PoiDescription { identifier },
            tickets: None,
            usable_distance: None,
            blocks,
            tags: None,
        }
    }

    /// How many entities can reserve a point of this type: `tickets`, or `0` when left out.
    #[must_use]
    pub fn tickets(&self) -> u8 {
        self.tickets.map_or(0, |tickets| tickets.0)
    }

    /// The distance a point is usable from: `usable_distance`, or `1` when left out.
    #[must_use]
    pub fn usable_distance(&self) -> u8 {
        self.usable_distance.map_or(1, |distance| distance.0)
    }

    /// The tags the type carries: `tags`, or none when left out.
    #[must_use]
    pub fn tags(&self) -> &[PoiTagId] {
        self.tags.as_deref().unwrap_or_default()
    }
}

impl PoiTag {
    /// A tag without broader or narrower tags.
    #[must_use]
    pub fn new(identifier: PoiTagId) -> PoiTag {
        PoiTag {
            description: PoiTagDescription { identifier },
            children: None,
            parents: None,
        }
    }

    /// The narrower tags: `children`, or none when left out.
    #[must_use]
    pub fn children(&self) -> &[PoiTagId] {
        self.children.as_deref().unwrap_or_default()
    }

    /// The broader tags: `parents`, or none when left out.
    #[must_use]
    pub fn parents(&self) -> &[PoiTagId] {
        self.parents.as_deref().unwrap_or_default()
    }
}
