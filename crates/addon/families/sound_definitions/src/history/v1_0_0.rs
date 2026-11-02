//! Format versions below 1.14.0, and every file without a readable `format_version`.

use super::{
    Diagnostics, EncodeError, FormatVersion, JsonSlice, MolangFields, MolangVisitor, ParseCx,
    Upgrade, UpgradeCx, Upgraded, Value, VersionStruct, Versioned, document, upgrade, v1_14_0,
};

/// The versionless shape: the root object is the map of definitions, each as written.
#[derive(Clone, Debug, PartialEq)]
pub struct SoundDefinitions {
    /// The root: one member per sound event name.
    pub definitions: Value,
}

impl Versioned for SoundDefinitions {
    const VERSION: FormatVersion = FormatVersion::new(1, 0, 0);
}

impl MolangFields for SoundDefinitions {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

impl VersionStruct for SoundDefinitions {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        document(payload, cx, d).map(|definitions| Self { definitions })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        Ok(self.definitions.clone())
    }
}

/// The 1.14.0 step: the root moves under `sound_definitions` of a root stamped
/// `"1.14.0"`; every definition gets `__use_legacy_max_distance: "true"` and a `sounds`
/// member, and the object entries of `sounds` lose their `pitch`. A root that is no object
/// is left as it is.
impl Upgrade for SoundDefinitions {
    type Next = v1_14_0::SoundDefinitions;

    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Self::Next {
        let mut document = self.definitions;
        if document.is_object() {
            upgrade::wrap_legacy_root(&mut document, &mut Upgraded::default());
        }
        v1_14_0::SoundDefinitions { document }
    }
}
