//! `minecraft:structure_set`: a structure set.

use super::validate::validate_detached;
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::{JigsawStructureId, StructureSetId};
use bedrock_addon_core::json::StrictInt;
use bedrock_addon_core::load::LoadContext;

/// A structure set: jigsaw structures placed by one placement rule (`minecraft:structure_set`,
/// in `worldgen/structure_sets/`).
///
/// JSON: `{"format_version": "1.21.20", "minecraft:structure_set": {"description",
/// "placement", "structures"}}`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[doc(alias = "minecraft:structure_set")]
#[doc(alias = "JigsawStructureSet")]
pub struct StructureSet {
    /// The set's identifier (JSON `description`).
    pub description: StructureSetDescription,
    /// Where the set's structures are placed (JSON `placement`).
    pub placement: RandomSpreadPlacement,
    /// The structures and their weights (JSON `structures`).
    pub structures: Vec<StructureSetEntry>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:structure_set/description`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct StructureSetDescription {
    /// The set's identifier, `namespace:name` (JSON `identifier`).
    pub identifier: StructureSetId,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The placement of a structure set: one attempt per grid cell of `spacing` chunks, at least
/// `separation` chunks from the cell's edge; `spacing` must be more than twice `separation`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct RandomSpreadPlacement {
    /// The placement type, only `minecraft:random_spread` (JSON `type`).
    #[facet(rename = "type")]
    pub placement_type: PlacementType,
    /// Seeds the placement (JSON `salt`).
    pub salt: StrictInt<i32>,
    /// The minimum distance in chunks between two placements (JSON `separation`).
    pub separation: StrictInt<u16>,
    /// The grid size in chunks, 1 or more (JSON `spacing`).
    pub spacing: StrictInt<u16>,
    /// How the offset inside a cell is drawn (JSON `spread_type`, default `linear`).
    #[facet(default)]
    pub spread_type: Option<SpreadType>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The placement type of a structure set (`placement.type`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, facet::Facet)]
#[repr(u8)]
#[non_exhaustive]
pub enum PlacementType {
    /// JSON `minecraft:random_spread`.
    #[default]
    #[facet(rename = "minecraft:random_spread")]
    RandomSpread,
}

/// How a placement's offset inside its cell is drawn (`spread_type`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, facet::Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum SpreadType {
    /// Uniformly (JSON `linear`), the default.
    #[default]
    Linear,
    /// Towards the cell's centre (JSON `triangular`).
    Triangular,
}

/// One structure of a set (`structures[]`).
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct StructureSetEntry {
    /// The jigsaw structure (JSON `structure`).
    pub structure: JigsawStructureId,
    /// Its weight, 1 to 200 (JSON `weight`).
    pub weight: StrictInt<u8>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl StructureSet {
    /// A set without structures.
    pub fn new(identifier: StructureSetId, placement: RandomSpreadPlacement) -> StructureSet {
        StructureSet {
            description: StructureSetDescription {
                identifier,
                extra: Extra::default(),
            },
            placement,
            structures: Vec::new(),
            extra: Extra::default(),
        }
    }

    /// Runs the family's parse constraints and registration checks as if the set were loaded
    /// from a behaviour pack with `cx`. The set is not changed.
    pub fn validate(&self, cx: &LoadContext) -> Diagnostics {
        validate_detached(self, cx)
    }
}

impl RandomSpreadPlacement {
    /// A random-spread placement with the default spread type.
    pub fn new(salt: i32, separation: u16, spacing: u16) -> RandomSpreadPlacement {
        RandomSpreadPlacement {
            placement_type: super::PlacementType::RandomSpread,
            salt: StrictInt(salt),
            separation: StrictInt(separation),
            spacing: StrictInt(spacing),
            spread_type: None,
            extra: Extra::default(),
        }
    }

    /// The spread type; `linear` when absent.
    pub fn spread_type(&self) -> SpreadType {
        self.spread_type.unwrap_or_default()
    }
}
