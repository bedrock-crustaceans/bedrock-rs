//! Reading and writing [`ClientBiome`] and [`Beta`] as version structs: the payload object,
//! and the document with its envelope.

use facet_value::VObject;

use super::component::ClientBiomeComponent;
use super::history::ClientBiomeFamily;
use super::read::{ComponentEra, MODERN, PAYLOAD_KEY, beta_payload, payload, strict};
use super::{Beta, ClientBiome, Description};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::{ComponentMap, UnknownMembers};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value, WriteStyle, write_json};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BETA_VERSION, ParseCx, VersionStruct, Versioned, write_envelope,
};
use bedrock_addon_shared_biome_env::{beta_object, payload_object, write_components};

/// The payload object: `description` and `components`; the writer adds the document's unknown
/// members after them.
pub(crate) fn object<C: ComponentEra>(
    description: &Description,
    map: &ComponentMap<C>,
) -> Result<VObject, EncodeError> {
    // Keys no table knows are written back as they were read.
    payload_object(description, write_components(map, C::write)?)
}

/// The whole document: `format_version` and the payload (the writer adds the root's unknown
/// members from the document's table). The
/// legacy structs' text is the whole document (the raw-text path, below 1.21.130) with the
/// version as written, so they write their envelope here; the newest and the beta struct go
/// through core's `write_envelope`.
pub(crate) fn document(version: &str, payload: Value) -> Value {
    let mut root = VObject::new();
    root.insert("format_version", Value::from(version));
    root.insert(PAYLOAD_KEY, payload);
    root.into()
}

impl Versioned for ClientBiome {
    const VERSION: FormatVersion = MODERN;
}

impl VersionStruct for ClientBiome {
    fn parse(slice: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let payload = strict(cx, d, |d| {
            payload::<ClientBiomeComponent>(slice, Self::VERSION, cx, d)
        })?;
        Some(Self {
            description: payload.description,
            components: payload.components,
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        Ok(object(&self.description, &self.components)?.into())
    }
}

impl Versioned for Beta {
    const VERSION: FormatVersion = BETA_VERSION;
}

impl VersionStruct for Beta {
    fn parse(slice: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let (payload, environment_attributes) = strict(cx, d, |d| beta_payload(slice, cx, d))?;
        Some(Self {
            description: payload.description,
            components: payload.components,
            environment_attributes,
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        // `environment_attributes` follows `components`, before the unknown members.
        beta_object(
            &self.description,
            write_components(
                &self.components,
                <ClientBiomeComponent as ComponentEra>::write,
            )?,
            self.environment_attributes.as_ref(),
        )
    }
}

impl ClientBiome {
    /// Writes the document, at format 1.21.130, into `out`.
    pub(crate) fn write_into(
        &self,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<(), EncodeError> {
        let root = write_envelope::<ClientBiomeFamily>(
            Self::VERSION,
            VersionStruct::write(self)?,
            &UnknownMembers::default(),
        );
        write_json(&root, out, style)
    }
}

impl Beta {
    /// Writes the document, at `"format_version": "beta"`, into `out`.
    pub(crate) fn write_into(
        &self,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<(), EncodeError> {
        let root = write_envelope::<ClientBiomeFamily>(
            BETA_VERSION,
            VersionStruct::write(self)?,
            &UnknownMembers::default(),
        );
        write_json(&root, out, style)
    }
}
