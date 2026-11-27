//! Format versions from 1.14.0 up to 1.20.20.

use super::{
    Diagnostics, EncodeError, FormatVersion, JsonSlice, MolangFields, MolangVisitor, ParseCx,
    Upgrade, UpgradeCx, Upgraded, Value, VersionStruct, Versioned, document, upgrade, v1_20_20,
};

/// The envelope `{ format_version, sound_definitions }`, with integer distances still
/// ignored.
#[derive(Clone, Debug, PartialEq)]
pub struct SoundDefinitions {
    /// The whole document.
    pub document: Value,
}

impl Versioned for SoundDefinitions {
    const VERSION: FormatVersion = FormatVersion::new(1, 14, 0);
}

impl MolangFields for SoundDefinitions {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

impl VersionStruct for SoundDefinitions {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        document(payload, cx, d).map(|document| Self { document })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        Ok(self.document.clone())
    }
}

/// The 1.20.20 step: the root is stamped `"1.20.20"`, and every definition has its integer
/// `max_distance` and `min_distance` replaced by `0.0` and the absent ones inserted as
/// `null`. A root that is no object is left as it is.
impl Upgrade for SoundDefinitions {
    type Next = v1_20_20::SoundDefinitions;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut document = self.document;
        if document.is_object() {
            upgrade::reset_integer_distances(
                &mut document,
                &mut Upgraded::default(),
                cx.diagnostics,
            );
        }
        v1_20_20::SoundDefinitions { document }
    }
}
