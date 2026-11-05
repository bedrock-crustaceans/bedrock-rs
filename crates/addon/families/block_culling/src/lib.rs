//! Block culling rules (`CullingRules`): `block_culling/**/*.json` of a resource pack
//! (`minecraft:block_culling_rules`), which parts of a block model are hidden against which
//! neighbour. A behaviour-pack block names a rules document with `minecraft:geometry` `culling`.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;

mod culling;
mod load;
mod new;
mod texts;
mod validate;
mod write;

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // the clock, to measure
mod cost;

use bedrock_addon_core::id::CullingRulesId;
use bedrock_addon_core::version::FormatVersion;

#[doc(hidden)]
pub use load::LOADER;

/// The culling rules of a block model (`minecraft:block_culling_rules`), at the newest format
/// version: which parts of the model are hidden against which neighbour.
///
/// JSON: `{"format_version": "1.21.80", "minecraft:block_culling_rules": {"description":
/// {"identifier"}, "rules": [{"geometry_part": {"bone", "cube"?, "face"?}, "direction",
/// "condition"?, "cull_against_full_and_opaque"?}]}}`. An unknown member fails the file. A
/// behaviour-pack block names a rules document with `minecraft:geometry` `culling`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[doc(alias = "minecraft:block_culling_rules")]
#[doc(alias = "block_culling")]
pub struct CullingRules {
    /// What the document is (JSON `description`).
    pub description: Description,
    /// The rules; the list may be empty (JSON `rules`).
    pub rules: Vec<Rule>,
    /// The document's own `format_version`. Not a JSON member; the writer writes 1.21.80, the
    /// only version whose document holds every member.
    pub source_format_version: FormatVersion,
}

/// `minecraft:block_culling_rules/description`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Description {
    /// The identifier a block's `minecraft:geometry` `culling` names (JSON `identifier`):
    /// `namespace:name`, without spaces, tabs or line breaks. Only a base-game pack may use
    /// the `minecraft` namespace.
    pub identifier: CullingRulesId,
}

/// One culling rule: a part of the model and the neighbour it is hidden against.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    /// When the part is hidden against a neighbour that is not a full, opaque block (JSON
    /// `condition`, from format 1.21.80); without one it never is.
    pub condition: Option<Condition>,
    /// Whether the part is hidden against a full, opaque neighbour (JSON
    /// `cull_against_full_and_opaque`, from format 1.21.80);
    /// [`culls_against_full_and_opaque`](Self::culls_against_full_and_opaque) gives the
    /// client's `true` when the member is absent.
    pub cull_against_full_and_opaque: Option<bool>,
    /// The part of the model (JSON `geometry_part`).
    pub geometry_part: GeometryPart,
    /// The side of the block whose neighbour decides (JSON `direction`).
    pub direction: Facing,
}

/// The part of a block model a rule hides: a whole bone, or one face of one cube of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeometryPart {
    /// The bone's name (JSON `bone`).
    pub bone: String,
    /// The index of a cube of the bone, 0 or more (JSON `cube`); it needs `face`.
    pub cube: Option<i32>,
    /// The face of that cube (JSON `face`); it needs `cube`.
    pub face: Option<Facing>,
}

/// A side of a block (vanilla `SharedTypes::Facing`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i16)]
pub enum Facing {
    /// `"down"`.
    Down = 0,
    /// `"up"`.
    Up = 1,
    /// `"north"`.
    North = 2,
    /// `"south"`.
    South = 3,
    /// `"west"`.
    West = 4,
    /// `"east"`.
    East = 5,
}

/// What a culling rule compares the neighbour with (JSON `condition`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
#[non_exhaustive]
pub enum Condition {
    /// `"same_culling_layer"`: the neighbour's `minecraft:geometry` `culling_layer` is defined
    /// and equal to this block's.
    SameCullingLayer = 1,
    /// `"same_block"`: the neighbour is the same block.
    SameBlock = 2,
    /// `"same_block_permutation"`: the neighbour is the same permutation of the same block.
    SameBlockPermutation = 3,
}

/// One rule as the client holds it after loading (vanilla `BlockCullingData::CullingRule`):
/// absent members are sentinels.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct CullingRule {
    /// The side whose neighbour is checked: [`Facing`]'s number, 0 to 5.
    pub neighbor_to_check: u8,
    /// The condition's number less one (`same_culling_layer` 0, `same_block` 1,
    /// `same_block_permutation` 2); `None` without a condition.
    pub condition: Option<u8>,
    /// Whether the part is hidden against a full, opaque neighbour.
    pub cull_against_full_and_opaque: bool,
    /// The bone's name.
    pub bone: String,
    /// The cube's index, or -1 for the whole bone.
    pub cube_index: i32,
    /// The face's number (0 to 5), or 6 for no face.
    pub face_to_cull: u8,
}

#[cfg(test)]
mod eq_tests {
    use super::CullingRules;

    fn assert_eq_trait<T: Eq>() {}

    #[test]
    fn culling_rules_is_eq() {
        assert_eq_trait::<CullingRules>();
        assert_eq_trait::<crate::history::v1_20_60::CullingRules>();
    }
}
