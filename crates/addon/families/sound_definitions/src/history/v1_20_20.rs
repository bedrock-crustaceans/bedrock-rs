//! Format versions from 1.20.20 up to 1.26.50.

use super::super::SoundDefinitions as Latest;
use super::{
    Diagnostics, EncodeError, FormatVersion, JsonSlice, MolangFields, MolangVisitor, ParseCx,
    Upgrade, UpgradeCx, Upgraded, Value, VersionStruct, Versioned, document, upgrade,
};

/// The envelope with integer distances read, and a streamed entry that does not say
/// streaming alone.
#[derive(Clone, Debug, PartialEq)]
pub struct SoundDefinitions {
    /// The whole document.
    pub document: Value,
}

impl Versioned for SoundDefinitions {
    const VERSION: FormatVersion = upgrade::V1_20_20;
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

/// The 1.26.50 step: the root is stamped `"1.26.50"`, and every object entry of a
/// definition's `sounds` list that streams and has no `allow_concurrent_streaming` gets
/// `allow_concurrent_streaming: false`. A root that is no object is left as it is.
impl Upgrade for SoundDefinitions {
    type Next = Latest;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Latest {
        let mut document = self.document;
        if document.is_object() {
            upgrade::disallow_concurrent_streaming(&mut document, &mut Upgraded::default());
        }
        Latest::read(document, cx.diagnostics)
    }
}
