//! The references of a jigsaw document: its template pools, processor lists and jigsaw
//! structures, and the feature of a pool element. What the documents name in other modules
//! (blocks, loot tables, structures) is in the pair files.

use super::super::{Findings, LinkCx};
use crate::families::jigsaw;
use crate::id::{
    BlockId, FeatureId, JigsawStructureId, LootTablePath, ProcessorListId, StructureId,
    TemplatePoolId,
};

/// The lookups of the pass, for the jigsaw checker. A target whose module is compiled out
/// counts as resolved: nobody can tell, and nothing is told.
#[derive(Debug)]
struct World<'c, 'a>(&'c LinkCx<'a>);

impl jigsaw::Targets for World<'_, '_> {
    fn block(&self, id: &BlockId) -> bool {
        #[cfg(feature = "block")]
        {
            super::jigsaw_block::jigsaw_block(self.0, id)
        }
        #[cfg(not(feature = "block"))]
        {
            let _ = id;
            true
        }
    }

    fn feature(&self, id: &FeatureId) -> bool {
        #[cfg(feature = "feature")]
        {
            !self.0.feature(id).is_missing()
        }
        #[cfg(not(feature = "feature"))]
        {
            let _ = id;
            true
        }
    }

    fn loot_table(&self, path: &LootTablePath) -> bool {
        #[cfg(feature = "loot")]
        {
            super::jigsaw_loot::jigsaw_loot_table(self.0, path)
        }
        #[cfg(not(feature = "loot"))]
        {
            let _ = path;
            true
        }
    }

    fn structure(&self, location: &StructureId) -> bool {
        #[cfg(feature = "structure")]
        {
            super::jigsaw_structure::jigsaw_template(self.0, location)
        }
        #[cfg(not(feature = "structure"))]
        {
            let _ = location;
            true
        }
    }

    fn structure_file(&self, location: &StructureId) -> bool {
        #[cfg(feature = "structure")]
        {
            super::jigsaw_structure::structure_file(self.0, location)
        }
        #[cfg(not(feature = "structure"))]
        {
            let _ = location;
            true
        }
    }

    // The checker has looked among the documents of the stack: what is left is the game's.
    fn pool(&self, id: &TemplatePoolId) -> bool {
        !self.0.template_pool(id).is_missing()
    }

    fn processor_list(&self, id: &ProcessorListId) -> bool {
        !self.0.processor_list(id).is_missing()
    }

    fn jigsaw_structure(&self, id: &JigsawStructureId) -> bool {
        !self.0.jigsaw_structure(id).is_missing()
    }
}

/// Every reference of a jigsaw document that resolves to nothing is an `UnresolvedReference`
/// finding at its member, once, with vanilla's text.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    out.extend(jigsaw::link(cx.behavior().jigsaw(), &World(cx)));
}
