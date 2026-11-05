//! The culling-rules types, their constructors, defaults and derived views.

use bedrock_addon_core::id::CullingRulesId;
use bedrock_addon_core::version::FormatVersion;

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

impl Facing {
    /// Every side, in vanilla's numbering.
    pub const ALL: [Facing; 6] = [
        Self::Down,
        Self::Up,
        Self::North,
        Self::South,
        Self::West,
        Self::East,
    ];

    /// The JSON name (`"north"`).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Down => "down",
            Self::Up => "up",
            Self::North => "north",
            Self::South => "south",
            Self::West => "west",
            Self::East => "east",
        }
    }

    /// The side a JSON name gives; names compare as written.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|facing| facing.name() == name)
    }

    /// Vanilla's number of the side, 0 to 5.
    #[must_use]
    pub const fn number(self) -> u8 {
        self as u8
    }

    /// The name vanilla prints in its messages (`Facing::toString`: `"East"`).
    pub(super) const fn title(self) -> &'static str {
        match self {
            Self::Down => "Down",
            Self::Up => "Up",
            Self::North => "North",
            Self::South => "South",
            Self::West => "West",
            Self::East => "East",
        }
    }
}

impl Condition {
    /// Every condition, in vanilla's numbering.
    pub const ALL: [Condition; 3] = [
        Self::SameCullingLayer,
        Self::SameBlock,
        Self::SameBlockPermutation,
    ];

    /// The JSON name (`"same_block"`).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::SameCullingLayer => "same_culling_layer",
            Self::SameBlock => "same_block",
            Self::SameBlockPermutation => "same_block_permutation",
        }
    }

    /// The condition a JSON name gives; names compare as written.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|condition| condition.name() == name)
    }
}

impl Rule {
    /// A rule hiding `geometry_part` against the neighbour on `direction`, without a condition.
    #[must_use]
    pub fn new(geometry_part: GeometryPart, direction: Facing) -> Self {
        Self {
            condition: None,
            cull_against_full_and_opaque: None,
            geometry_part,
            direction,
        }
    }

    /// Whether the part is hidden against a full, opaque neighbour: the member, or `true`.
    // The game's default rule stores 1 for this flag.
    #[must_use]
    pub fn culls_against_full_and_opaque(&self) -> bool {
        self.cull_against_full_and_opaque.unwrap_or(true)
    }

    /// The rule as the client holds it.
    #[must_use]
    pub fn to_client_rule(&self) -> CullingRule {
        CullingRule {
            neighbor_to_check: self.direction.number(),
            condition: self.condition.map(|condition| condition as u8 - 1),
            cull_against_full_and_opaque: self.culls_against_full_and_opaque(),
            bone: self.geometry_part.bone.clone(),
            cube_index: self.geometry_part.cube.unwrap_or(-1),
            face_to_cull: self.geometry_part.face.map_or(6, Facing::number),
        }
    }
}

impl GeometryPart {
    /// The whole bone `bone`.
    #[must_use]
    pub fn whole_bone(bone: impl Into<String>) -> Self {
        Self {
            bone: bone.into(),
            cube: None,
            face: None,
        }
    }

    /// The face `face` of cube `cube` of the bone `bone`.
    #[must_use]
    pub fn cube_face(bone: impl Into<String>, cube: i32, face: Facing) -> Self {
        Self {
            bone: bone.into(),
            cube: Some(cube),
            face: Some(face),
        }
    }
}

impl CullingRules {
    /// The newest format version: what [`CullingRules::new`] documents carry and the writer
    /// writes.
    pub const FORMAT_VERSION: FormatVersion = FormatVersion::new(1, 21, 80);

    /// Rules named `identifier`, with `rules`.
    #[must_use]
    pub fn new(identifier: CullingRulesId, rules: Vec<Rule>) -> Self {
        Self {
            description: Description { identifier },
            rules,
            source_format_version: Self::FORMAT_VERSION,
        }
    }

    /// Every rule as the client holds it, in order.
    #[must_use]
    pub fn to_client_rules(&self) -> Vec<CullingRule> {
        self.rules.iter().map(Rule::to_client_rule).collect()
    }
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
