//! Reading and writing [`ClientBiome`] and [`Beta`] as version structs: the payload object,
//! and the document with its envelope.

use facet_value::VObject;

use super::component::ClientBiomeComponent;
use super::history::ClientBiomeFamily;
use super::read::{ComponentEra, MODERN, PAYLOAD_KEY, beta_payload, payload, strict};
use super::{Beta, ClientBiome, Description};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::{ComponentMap, ComponentRef, Extra};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value, WriteStyle, write_json};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BETA_VERSION, ParseCx, VersionStruct, Versioned, write_envelope, write_payload,
};

/// A component map as a JSON object, in map order; keys no table knows are written back as
/// they were read.
fn components<C: ComponentEra>(map: &ComponentMap<C>) -> Result<Value, EncodeError> {
    let mut object = VObject::new();
    for entry in map.iter() {
        let (name, json) = match entry {
            ComponentRef::Typed { name, value, .. } => (name.to_owned(), value.write()?),
            ComponentRef::Custom(c) => (c.name.clone(), c.parameters.clone()),
            ComponentRef::Dynamic { name, json } | ComponentRef::Unknown { name, json } => {
                (name.to_owned(), json.clone())
            }
            _ => continue,
        };
        object.insert(name, json);
    }
    Ok(object.into())
}

/// The payload object: `description`, `components`, then the unknown members.
pub(crate) fn object<C: ComponentEra>(
    description: &Description,
    map: &ComponentMap<C>,
    extra: &Extra,
) -> Result<VObject, EncodeError> {
    let mut object = VObject::new();
    object.insert("description", write_payload(description)?);
    object.insert("components", components(map)?);
    for (key, value) in extra.iter() {
        object.insert(key, value.clone());
    }
    Ok(object)
}

/// The whole document: `format_version`, the payload, then the root's unknown members. The
/// legacy structs' text is the whole document (the raw-text path, below 1.21.130) with the
/// version as written, so they write their envelope here; the newest and the beta struct go
/// through core's `write_envelope`.
pub(crate) fn document(version: &str, payload: Value, root_extra: &Extra) -> Value {
    let mut root = VObject::new();
    root.insert("format_version", Value::from(version));
    root.insert(PAYLOAD_KEY, payload);
    for (key, value) in root_extra.iter() {
        root.insert(key, value.clone());
    }
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
            extra: payload.extra,
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        Ok(object(&self.description, &self.components, &self.extra)?.into())
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
            extra: payload.extra,
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        // `environment_attributes` follows `components`, before the unknown members.
        let mut written = VObject::new();
        let object = object(&self.description, &self.components, &Extra::default())?;
        for (key, value) in &object {
            written.insert(key.as_str(), value.clone());
        }
        if let Some(attributes) = &self.environment_attributes {
            written.insert("environment_attributes", write_payload(attributes)?);
        }
        for (key, value) in self.extra.iter() {
            written.insert(key, value.clone());
        }
        Ok(written.into())
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
            &Extra::default(),
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
            &Extra::default(),
        );
        write_json(&root, out, style)
    }
}
