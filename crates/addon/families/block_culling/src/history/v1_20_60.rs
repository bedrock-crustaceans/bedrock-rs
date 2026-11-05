//! Format 1.20.60: the first culling-rules document, whose rules are a part and a direction.
//! Its step copies every member into the 1.21.80 document.

use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};

use super::super::read::{Slice, read};
use super::super::write;
pub use super::super::{Description, Facing, GeometryPart};

/// A culling-rules document of `[1.20.60, 1.21.80)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CullingRules {
    /// What the document is (JSON `description`).
    pub description: Description,
    /// The rules (JSON `rules`).
    pub rules: Vec<Rule>,
    /// The document's own `format_version`. Not a JSON member.
    pub source_format_version: FormatVersion,
}

/// One culling rule of format 1.20.60: the part is hidden against a full, opaque neighbour on
/// `direction`, and against nothing else.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    /// The part of the model (JSON `geometry_part`).
    pub geometry_part: GeometryPart,
    /// The side of the block whose neighbour decides (JSON `direction`).
    pub direction: Facing,
}

impl Versioned for CullingRules {
    const VERSION: FormatVersion = FormatVersion::new(1, 20, 60);
}

impl VersionStruct for CullingRules {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let payload = read(payload, Slice::V1_20_60, &cx.at, d)?;
        Some(Self {
            description: payload.description,
            rules: payload
                .rules
                .into_iter()
                .map(|rule| Rule {
                    geometry_part: rule.geometry_part,
                    direction: rule.direction,
                })
                .collect(),
            source_format_version: cx.original_version,
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        let rules: Vec<super::super::Rule> = self
            .rules
            .iter()
            .map(|rule| super::super::Rule::new(rule.geometry_part.clone(), rule.direction))
            .collect();
        Ok(write::payload(&self.description, &rules))
    }
}

/// The step to format 1.21.80: `description` and each rule's `geometry_part` and `direction`
/// move unchanged; `condition` starts absent and `cull_against_full_and_opaque` as `true`.
/// No finding.
// `v1_20_60 → v1_21_80`: a reflection copy; the target `Rule`'s constructor stores 1 at
// `cull_against_full_and_opaque`
impl Upgrade for CullingRules {
    type Next = super::super::CullingRules;

    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Self::Next {
        super::super::CullingRules {
            description: self.description,
            rules: self
                .rules
                .into_iter()
                .map(|rule| super::super::Rule {
                    condition: None,
                    cull_against_full_and_opaque: Some(true),
                    geometry_part: rule.geometry_part,
                    direction: rule.direction,
                })
                .collect(),
            source_format_version: self.source_format_version,
        }
    }
}
