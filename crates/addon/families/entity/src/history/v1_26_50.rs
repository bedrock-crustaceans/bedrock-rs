//! Format 1.26.50: the struct and the step to 1.26.60, the newest.
//!
//! The struct is the newest struct's shape read at 1.26.50: every component that 1.26.50 types
//! is read strictly, and the 31 keys vanilla first types at 1.26.60 (27 components, 4 goals) are
//! read through their lenient conversion (`history/typed.rs`), which is all the step does in the
//! game: the step only converts those 31 keys. The step itself therefore hands the value on.

use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};

use crate::{read, write};

/// An entity document as written in format 1.26.50 (up to 1.26.60): the newest struct's shape,
/// each component read by the version the document was written in.
#[derive(Clone, Debug, PartialEq)]
pub struct Entity(pub crate::Entity);

impl Versioned for Entity {
    const VERSION: FormatVersion = known::V1_26_50;
}

impl VersionStruct for Entity {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        read::entity(payload, cx, d).map(Self)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write::payload(&self.0)
    }
}

impl MolangFields for Entity {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        self.0.visit_molang(visitor);
    }
}

/// 1.26.50 → 1.26.60: the 31 first-typed conversions, done by the read of the struct (the
/// components the document's version does not type yet are read leniently).
impl Upgrade for Entity {
    type Next = crate::Entity;

    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Self::Next {
        self.0
    }
}
