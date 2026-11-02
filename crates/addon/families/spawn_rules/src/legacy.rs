//! The 1.8.0 shape of spawn rules: vanilla reads it with the hand-written parser over the raw
//! text that spawn groups share.

use super::{StandardDescription, StandardSpawnRules};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::id::SpawnRulesId;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned};
use bedrock_addon_support::spawn::legacy::{
    Flavor, RULES_KEY, V1_8_0, description_value, read, visit_conditions, write,
};

impl Versioned for StandardSpawnRules {
    const VERSION: FormatVersion = V1_8_0;
}

impl VersionStruct for StandardSpawnRules {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let read = read(payload, Flavor::Rules, d)?;
        Some(Self {
            format_version: read.has_version.then_some(cx.original_version),
            description: StandardDescription {
                identifier: SpawnRulesId::parse(read.identifier),
                population_control: read.population_control,
                extra: read.description_extra,
            },
            conditions: read.conditions,
            extra: read.extra,
            root_extra: read.root_extra,
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        Ok(self.to_value())
    }
}

impl StandardSpawnRules {
    /// The whole document as JSON.
    pub(crate) fn to_value(&self) -> Value {
        write(
            self.format_version,
            RULES_KEY,
            description_value(
                self.description.identifier.as_str(),
                self.description.population_control.as_deref(),
                &self.description.extra,
            ),
            self.conditions.as_deref(),
            &self.extra,
            &self.root_extra,
        )
    }
}

impl MolangFields for StandardSpawnRules {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visit_conditions(RULES_KEY, &mut self.conditions, visitor);
    }
}
