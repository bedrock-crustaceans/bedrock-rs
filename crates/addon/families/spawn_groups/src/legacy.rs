//! The 1.8.0 shape of spawn groups: vanilla reads it with the hand-written parser over the raw
//! text that standard spawn rules share.

use super::{GroupDescription, SpawnGroup};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::id::SpawnGroupId;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned};
use bedrock_addon_support::spawn::legacy::{
    Flavor, GROUPS_KEY, V1_8_0, description_value, read, visit_conditions, write,
};

impl Versioned for SpawnGroup {
    const VERSION: FormatVersion = V1_8_0;
}

impl VersionStruct for SpawnGroup {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let read = read(payload, Flavor::Groups, d)?;
        Some(Self {
            format_version: cx.original_version,
            description: GroupDescription {
                identifier: SpawnGroupId::parse(read.identifier),
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

impl SpawnGroup {
    /// The whole document as JSON.
    pub(crate) fn to_value(&self) -> Value {
        write(
            Some(self.format_version),
            GROUPS_KEY,
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

impl MolangFields for SpawnGroup {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visit_conditions(GROUPS_KEY, &mut self.conditions, visitor);
    }
}
