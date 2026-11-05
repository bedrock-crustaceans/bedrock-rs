//! Format 1.26.20: the first cereal document (`[1.26.20, 1.26.60)`). Its step is `upgrade_to_v1_26_60`: every
//! `minecraft:collision_box` box, of the base and of each permutation, becomes `min`/`max`.

use crate::component::{CollisionBoundingBox, CollisionBox};
use crate::read::{CEREAL, parse_document};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::ComponentMap;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};

/// A block document of `[1.26.20, 1.26.60)`, and the conversion of a legacy document: the
/// newest block's members, each collision box held as 1.26.20 reads it (`origin`/`size`).
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    /// The document. Its `minecraft:collision_box` values hold the boxes as written; one box
    /// written as an object is a list of one.
    pub block: crate::Block,
    /// The sections (`None` the base, `Some(i)` permutation `i`) whose collision box is one
    /// box written as an object, the 1.26.20 alternative the 1.26.60 component lacks.
    pub one_box: Vec<Option<usize>>,
}

impl Versioned for Block {
    const VERSION: FormatVersion = CEREAL;
}

impl VersionStruct for Block {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        parse_document(payload, cx, d).map(|(block, one_box)| Block { block, one_box })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        let mut payload = self.block.write_at(CEREAL)?;
        for section in &self.one_box {
            let components = match section {
                None => payload
                    .as_object_mut()
                    .and_then(|payload| payload.get_mut("components")),
                Some(index) => payload
                    .as_object_mut()
                    .and_then(|payload| payload.get_mut("permutations"))
                    .and_then(Value::as_array_mut)
                    .and_then(|permutations| permutations.get_mut(*index))
                    .and_then(Value::as_object_mut)
                    .and_then(|permutation| permutation.get_mut("components")),
            };
            if let Some(collision) = components
                .and_then(Value::as_object_mut)
                .and_then(|components| components.get_mut("minecraft:collision_box"))
                && let Some(first) = collision
                    .as_array()
                    .filter(|boxes| boxes.len() == 1)
                    .and_then(|boxes| boxes.first())
                    .cloned()
            {
                *collision = first;
            }
        }
        Ok(payload)
    }
}

impl MolangFields for Block {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        self.block.visit_molang(visitor);
    }
}

/// `upgrade_to_v1_26_60`: converts the base's and each permutation's
/// `minecraft:collision_box`; `true`/`false` stay, each box becomes `min`/`max` (see
/// [`CollisionBoundingBox::from_origin_size`](crate::component::CollisionBoundingBox::from_origin_size)),
/// the x axis mirrored. Silent: the game's step logs nothing.
impl Upgrade for Block {
    type Next = crate::Block;

    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Self::Next {
        let mut block = self.block;
        upgrade_section(&mut block.components);
        for permutation in &mut block.permutations {
            upgrade_section(&mut permutation.components);
        }
        block
    }
}

/// One section's collision box, upgraded.
fn upgrade_section(map: &mut ComponentMap<crate::component::BlockComponent>) {
    let Some(CollisionBox::Boxes(boxes)) = map.get_mut::<CollisionBox>() else {
        return;
    };
    for b in boxes.iter_mut() {
        let mut upgraded = CollisionBoundingBox::from_origin_size(b.origin(), b.size());
        upgraded.extra = std::mem::take(&mut b.extra);
        *b = upgraded;
    }
}
