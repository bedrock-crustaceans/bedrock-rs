//! Constructors, defaults and the derived views of the culling-rules types.

use super::{Condition, CullingRule, CullingRules, Description, Facing, GeometryPart, Rule};
use bedrock_addon_core::id::CullingRulesId;
use bedrock_addon_core::version::FormatVersion;

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
